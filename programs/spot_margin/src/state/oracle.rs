use anchor_lang::prelude::*;

use crate::{
    error::SpedXSpotResult,
    math::{
        casting::Cast,
        constants::{
            PRICE_PRECISION,
            PRICE_PRECISION_I64,
            PRICE_PRECISION_U64
        },
        safe_math::SafeMath
    },
};

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
    /// Default implementation of HistoricalIndexData where values are represented by quote values in PRICE_PRECISION.
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
    pub has_sufficient_data_points: bool
}