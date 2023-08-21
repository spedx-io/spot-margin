#![allow(unused_imports)]

use anchor_lang::prelude::*;
use bytemuck::{cast_slice_mut, checked::{try_cast_slice_mut, from_bytes_mut}, Pod, Zeroable};
// use switchboard_v2::AggregatorAccountData;
use std::{cell::RefMut, str::FromStr};

use crate::{
    error::{SpedXSpotResult, ErrorCode},
    math::{
        casting::Cast,
        constants::{
            PRICE_PRECISION,
            PRICE_PRECISION_I64,
            PRICE_PRECISION_U64,
            STALENESS_THRESHOLD,
            TEN_BPS_I64
        },
        safe_math::SafeMath
    },
    state::{
        enums::OracleType,
        helpers::{
            get_test_pyth_price,
            get_account_bytes,
            create_account_info
        }
    },
    create_account_info
};
use bytes::BytesMut;

pub mod pyth_program {
    use solana_program::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
}

#[derive(Default, AnchorSerialize, AnchorDeserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub struct HistoricalPriceData {
    /// The most recent price provided by the Oracle, represented in PRICE_PRECISION
    pub last_oracle_price_data: i64,

    /// The most recent confidence interval provided by the Oracle, represented in PRICE_PRECISION
    pub last_oracle_conf: u64,

    /// The delay between the most recent price and the one before it
    pub last_oracle_delay: i64,

    /// TWAP of the most recent price provided by the Oracle, represented in PRICE_PRECISION
    pub last_oracle_twap: i64,

    /// 5min TWAP of the most recent price provided by the Oracle, represented in PRICE_PRECISION
    pub last_oracle_twap_5min: i64,

    /// TWAP of an arbitrary timestamp of the most recent price provided by the Oracle, represented in PRICE_PRECISION
    pub last_oracle_twap_time_stamp: i64,
}

impl HistoricalPriceData {

    /// Default implementation of HistoricalPriceData where fields are represented by quote values in PRICE_PRECISION.
    pub fn default_quote_oracle() -> Self {
        HistoricalPriceData { 
            last_oracle_price_data: PRICE_PRECISION_I64, 
            last_oracle_conf: 0, 
            last_oracle_delay: 0, 
            last_oracle_twap: PRICE_PRECISION_I64, 
            last_oracle_twap_5min: PRICE_PRECISION_I64, 
            ..HistoricalPriceData::default()
        }
    }

    /// Default implementation of HistoricalPriceData with default prices
    pub fn default_price(price: i64) -> Self { 
        HistoricalPriceData {
            last_oracle_price_data: price,
            last_oracle_conf: 0,
            last_oracle_delay: 0,
            last_oracle_twap: price,
            last_oracle_twap_5min: price,
            ..HistoricalPriceData::default()
        }
    }

    /// Default implementation of HistoricalPriceData with data from current oracle prices
    pub fn default_with_current_oracle(oracle_price_data: &OraclePriceData) -> Self {
        HistoricalPriceData {
            last_oracle_price_data: oracle_price_data.price,
            last_oracle_conf: oracle_price_data.confidence,
            last_oracle_delay: oracle_price_data.delay,
            last_oracle_twap: oracle_price_data.price,
            last_oracle_twap_5min: oracle_price_data.price,
            ..HistoricalPriceData::default()
        }
    }

}

#[derive(Default, AnchorSerialize, AnchorDeserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub struct HistoricalIndexData {
    /// The most recent best bid price of the index asset, represented in PRICE_PRECISION
    pub last_index_bid_price: u64,

    /// The most recent best ask price of the index asset, represented in PRICE_PRECISION
    pub last_index_ask_price: u64,

    /// The most recent twap of the index asset, represented in PRICE_PRECISION
    pub last_index_price_twap: u64,

    /// The most recent 5min twap of the index asset, represented in PRICE_PRECISION
    pub last_index_price_twap_5min: u64,

    pub last_index_price_twap_time_stamp: i64
}

impl HistoricalIndexData {
    pub fn default_quote_oracle() -> Self { 
        HistoricalIndexData {
            last_index_ask_price: PRICE_PRECISION_U64,
            last_index_bid_price: PRICE_PRECISION_U64,
            last_index_price_twap: PRICE_PRECISION_U64,
            last_index_price_twap_5min: PRICE_PRECISION_U64,
            ..HistoricalIndexData::default()
        }
    }

    /// Default implementation of HistoricalIndexData with current oracle values
    pub fn default_with_current_oracle(oracle_price_data: &OraclePriceData) -> SpedXSpotResult<Self> {
        let price = oracle_price_data.price.cast::<u64>().unwrap();

        Ok(
            HistoricalIndexData {
                last_index_bid_price: price,
                last_index_ask_price: price,
                last_index_price_twap: price,
                last_index_price_twap_5min: price,
                ..HistoricalIndexData::default()
            }
        )
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct OraclePriceData {
    /// Price of the base asset quoted in quote asset
    pub price: i64,
    /// Confidence interval of the base asset as provided by the oracle
    pub confidence: u64,
    /// Delay in provision of the current price in comparison to the one preceding it.
    pub delay: i64,
    /// Whether the data aggregated has sufficient individual data points.
    pub has_sufficient_data_points: bool,
    // pub ema: i64,
}

impl OraclePriceData {
    /// A default oracle quoting prices in usd
    pub fn default_usd() -> Self {
        OraclePriceData {
            price: PRICE_PRECISION_I64,
            confidence: 1,
            delay: 0,
            has_sufficient_data_points: true,
            // ema: PRICE_PRECISION_I64
        }
    }
}

pub fn get_pyth_price(
    price_oracle: &AccountInfo,
    clock_slot: u64,
    multiple: u128
) -> SpedXSpotResult<OraclePriceData> {
    // fetching price feed from pyth
    let price_feed = pyth_sdk_solana::load_price_feed_from_account_info(price_oracle).unwrap();

    // getting current timestamp using solana_program::Clock
    let curr_timestamp = Clock::get().unwrap().unix_timestamp;

    // getting current price from the price feed, and using a staleness parameter to it.
    let curr_price = price_feed.get_price_no_older_than(curr_timestamp, STALENESS_THRESHOLD).unwrap();

    let curr_price_ema = price_feed.get_ema_price_no_older_than(curr_timestamp, STALENESS_THRESHOLD).unwrap();

    msg!("Current price: {}, Current price EMA: {}, Current confidence interval: {}", curr_price.price, curr_price_ema.price, curr_price.conf);

    // setting the oracle precision to 10^price exponent
    let oracle_precision = 10_u128.pow(curr_price.expo.unsigned_abs());

    if oracle_precision <= multiple {
        msg!("Oracle multiple/exponent is higher than supported oracle precision");
        return Err(ErrorCode::InvalidOracle);
    }

    // getting the new oracle precision by divding the current oracle precision by the multiple
    let oracle_precision = oracle_precision.safe_div(multiple).unwrap();

    let mut oracle_scale_div = 1;
    let mut oracle_scale_mul = 1;

    // if the oracle precision is higher than the PRICE_PRECISION, find the ratio of the oracle precision with the
    // price precision
    if oracle_precision > PRICE_PRECISION {
        oracle_scale_div = oracle_precision.safe_div(PRICE_PRECISION)?;
    } else {
        oracle_scale_mul = PRICE_PRECISION.safe_div(oracle_precision)?;
    }

    // scaled oracle price
    let oracle_price_scaled = (curr_price.price)
        .cast::<i128>()?
        .safe_mul(oracle_scale_mul.cast()?)?
        .safe_div(oracle_scale_div.cast()?)?
        .cast::<i64>()?;

    // scaled oracle confidence interval
    let oracle_conf_scaled = (curr_price.conf)
        .cast::<u128>()?
        .safe_mul(oracle_scale_mul)?
        .safe_div(oracle_scale_div)?
        .cast::<u64>()?;

    // fetching delay between oracle updates
    let oracle_delay = clock_slot.cast::<i64>()?.safe_sub(curr_price.publish_time.cast()?)?;

    Ok(OraclePriceData {
        price: oracle_price_scaled,
        confidence: oracle_conf_scaled,
        delay: oracle_delay,
        has_sufficient_data_points: true,
        // ema: oracle_ema_price_scaled
    })
}

pub fn get_pyth_stablecoin_price(
    price_oracle: &AccountInfo,
    clock_slot: u64,
    // multiple: u128
) -> SpedXSpotResult<OraclePriceData> {
    let multiple = 1;
    let mut oracle_price_data = get_pyth_price(price_oracle, clock_slot, multiple)?;

    let price = oracle_price_data.price;
    // let price_u64 = price as u64;

    let confidence = oracle_price_data.confidence;
    
    let five_bps = TEN_BPS_I64/2;

    // if the difference between the price and price precision, where the price precision is a price value, is
    // is less than 5 bps, then set the price to the price precision
    if price.safe_sub(PRICE_PRECISION_I64)?.abs() <= five_bps.min(confidence.cast()?) {
        oracle_price_data.price = PRICE_PRECISION_I64;
    }

    Ok(oracle_price_data)
}

pub fn get_oracle_price(
    oracle_type: &OracleType,
    price_oracle: &AccountInfo,
    clock_slot: u64
) -> SpedXSpotResult<OraclePriceData> {
    let multiple = 1;
    let multiple1k = 1000;
    let multiple1m = 1000000;
    match oracle_type {
        OracleType::Pyth => get_pyth_price(price_oracle, clock_slot, multiple),
        OracleType::Pyth1K => get_pyth_price(price_oracle, clock_slot, multiple1k),
        OracleType::Pyth1M => get_pyth_price(price_oracle, clock_slot, multiple1m),
        OracleType::PythStables => get_pyth_stablecoin_price(price_oracle, clock_slot),
        OracleType::Switchboard => {
            msg!("Switchboard oracles not implemented yet. Stay tuned!");
            return Err(ErrorCode::InvalidOracle);
        }
        OracleType::QuoteAsset => Ok(OraclePriceData {
            price: PRICE_PRECISION_I64,
            confidence: 1,
            delay: 0,
            has_sufficient_data_points: true,
            // ema: PRICE_PRECISION_I64
        })
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Price { 
    pub pyth_magic_number: u32,
    pub pyth_program_version: u32,
    pub atype: u32,
    pub size: u32,
    pub price_type: crate::state::enums::PriceType,
    pub exponent: i32,
    pub num: u32,
    pub curr_slot: u64,
    pub curr_valid_slot: u64,
    pub twap: i64,
    pub price_vol: u64,
    pub acc_key: PriceAccountKey,
    pub next_acc_key: PriceAccountKey,
    pub aggregator_pubkey: PriceAccountKey,
    pub aggregator: PriceInfo,
    pub details: [PriceDetails;3]
}

impl Price {
    #[inline]
    pub fn load_prices<'a>(price_feed: &'a AccountInfo) -> std::result::Result<RefMut<'a, Price>, ProgramError> {
        let data: RefMut<'a, [u8]> =
            RefMut::map(price_feed.try_borrow_mut_data().unwrap(), |data| *data);

        let state: RefMut<'a, Self> = RefMut::map(data, |f| {
            from_bytes_mut(cast_slice_mut::<u8, u8>(try_cast_slice_mut(f).unwrap()))
        });
        Ok(state)
    }
}

#[cfg(target_endian="little")]
unsafe impl Pod for Price {}

#[cfg(target_endian="little")]
unsafe impl Zeroable for Price {}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PriceAccountKey {
    pub val: [u8;32]
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PriceInfo {
    pub price: i64,
    pub conf: u64,
    pub status: PriceStatus,
    pub slot: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum PriceStatus {
    NotInitialized,
    Tradeable,
    Halted
}

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Tradeable
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PriceDetails {
    publisher: Pubkey,
    aggregator: PriceAccountKey,
    latest_price: PriceInfo,
}

/// Test to get pyth price at a multiple/precision of 1,000
/// We use this multiple upon the oracle prices to obtain a precise oracle price
#[test]
fn pyth_1k() {
    let mut oracle_price = get_test_pyth_price(8394, 10);
    let oracle_price_key =
        Pubkey::from_str("8ihFLu5FimgTQ1Unh4dVyEHUGodJ5gJQCrQf4KUVB9bN").unwrap();
    let pyth_program = crate::state::oracle::pyth_program::id();
    create_account_info!(
        oracle_price,
        &oracle_price_key,
        &pyth_program,
        oracle_account_info
    );

    let oracle_price_data =
        get_oracle_price(&OracleType::Pyth1K, &oracle_account_info, 0).unwrap();
    assert_eq!(oracle_price_data.price, 839);
}

/// Test to get pyth price at a multiple/precision of 1,000,000
/// We use this multiple upon the oracle prices to obtain a precise oracle price
#[test]
fn pyth_1m() {
    let mut oracle_price = get_test_pyth_price(8394, 10);
    let oracle_price_key =
        Pubkey::from_str("8ihFLu5FimgTQ1Unh4dVyEHUGodJ5gJQCrQf4KUVB9bN").unwrap();
    let pyth_program = crate::state::oracle::pyth_program::id();
    create_account_info!(
        oracle_price,
        &oracle_price_key,
        &pyth_program,
        oracle_account_info
    );

    let oracle_price_data =
        get_oracle_price(&OracleType::Pyth1M, &oracle_account_info, 0).unwrap();
    assert_eq!(oracle_price_data.price, 839400);
}

/// Macro that can be called to create account info using available data such as
/// - Account Pubkey
/// - lamports
/// - account_bytes(which represents the account in deserialized form)
/// - owner of the account
#[macro_export]
macro_rules! create_account_info {
    ($account:expr, $owner:expr, $name: ident) => {
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = get_account_bytes(&mut $account);
        let owner = $type::owner();
        let $name = create_account_info(&key, true, &mut lamports, &mut data[..], $owner);
    };
    ($account:expr, $pubkey:expr, $owner:expr, $name: ident) => {
        let mut lamports = 0;
        let mut data = get_account_bytes(&mut $account);
        let $name = create_account_info($pubkey, true, &mut lamports, &mut data[..], $owner);
    };
}