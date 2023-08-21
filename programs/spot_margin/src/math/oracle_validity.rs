#![allow(unused_imports)]

use std::cmp::max;
use crate::{
    state::{
        config::State,
        guard_rails::{
            OracleGuardRails,
            ValidityGuardRails,
            PriceDivergenceGuardRails
        },
    },
    math::constants::{
        PRICE_PRECISION,
        PRICE_PRECISION_U64
    }
};

use crate::{
    error::{
        SpedXSpotResult,
        ErrorCode
    },
    math::{
        casting::Cast,
        constants::{
            BID_ASK_SPREAD_PRECISION,
            PERCENTAGE_PRECISION_U64,
            BID_ASK_SPREAD_PRECISION_I128
        },
        safe_math::SafeMath,
    },
    state::{
        oracle::{OraclePriceData, HistoricalPriceData, get_pyth_price},
        enums::{
            MarketStatus,
            OracleValidity,
            Actions
        },
        market::Market
    }
};

/// Function to check if the oracle data is valid for a specific action in the protocol
/// Such as filling an order, adding an order, etc.
pub fn is_oracle_valid_for_action(
    action: Option<Actions>,
    validity: OracleValidity,
) -> SpedXSpotResult<bool> {
    let is_action_valid = match action {
        Some(action) => match action {
            Actions::FillOrder => matches!(validity, OracleValidity::Valid),
            // by optimizing for the most valid price for margin calculation, we ensure that prices are always up-to-date
            // and valid
            Actions::MarginCalculation => !matches!( 
                validity,
                OracleValidity::Invalid | OracleValidity::Volatile | OracleValidity::Uncertain | OracleValidity::StaleForMargin
                | OracleValidity::InsufficientDataPoints
            ),
            Actions::OrderAdded => !matches!(
                validity,
                OracleValidity::Invalid | OracleValidity::Volatile | OracleValidity::Uncertain
            ),
            Actions::PnLSettlement => !matches!(
                validity,
                OracleValidity::Invalid | OracleValidity::Volatile | OracleValidity::Uncertain
            ),
            // by optimizing for the most valid price for twap update, we ensure that prices are always up-to-date
            // and valid
            Actions::UpdateTWAP => !matches!(
                validity,
                OracleValidity::Invalid | OracleValidity::Volatile | OracleValidity::Uncertain | OracleValidity::InsufficientDataPoints
            ),
            // by optimizing for the most valid price for liquidations, we ensure that prices are always up-to-date
            // and valid
            Actions::Liquidate => !matches!(
                validity,
                OracleValidity::Invalid | OracleValidity::Volatile | OracleValidity::Uncertain | OracleValidity::InsufficientDataPoints
            ),
        },
        None => {
            matches!(
                validity,
                OracleValidity::Valid
            )
        }
    };

    Ok(is_action_valid)
}

/// Function to check the oracle validity. Returns the OracleValidity enum, with different outcomes ranging from descending
/// order of severity of invalidity. 
pub fn oracle_validity(
    last_oracle_twap: i64,
    oracle_price_data: &OraclePriceData,
    oracle_validity_guard: &ValidityGuardRails
) -> SpedXSpotResult<OracleValidity> {
    let OraclePriceData {
        price: oracle_price,
        confidence: oracle_conf,
        delay: oracle_delay,
        has_sufficient_data_points,
        // ema: _oracle_ema
    } = *oracle_price_data;

    // Checking if the oracle price can ever be -ve, if yes, the token just got rekt
    let is_oracle_price_negative = oracle_price <= 0;

    if is_oracle_price_negative {
        return Err(ErrorCode::OracleNegativeError.into())
    }

    // We first check which is the greater value of the oracle price and its last available TWAP.
    // Then we divide it by the smaller value of the oracle price and its last available TWAP.
    // If this value is lesser than the max volatility ratio, then this variable returns false.
    // Else, it returns true.
    let is_oracle_price_too_volatile = (oracle_price.max(last_oracle_twap))
        .safe_div(last_oracle_twap.min(oracle_price).max(1))?
        .gt(&oracle_validity_guard.max_volatility_ratio);

    if is_oracle_price_too_volatile {
        return Err(ErrorCode::OracleTooVolatile.into())
    }

    // We try to calculate the confidence interval as a % of the oracle price. 
    // We do so by dividing the oracle confidence by the oracle price and multiplying it with BID_ASK_PRECISION which is 100.
    let confidence_as_pct_of_price = max(1, oracle_conf)
        .safe_mul(BID_ASK_SPREAD_PRECISION)?
        .safe_div(oracle_price.cast()?)?;

    // Returns true if the confidence interval as % of price is > max allowed %
    let is_confidence_too_wide = confidence_as_pct_of_price.gt(
        &oracle_validity_guard.confidence_interval_max_accepted_divergence
    );

    if is_confidence_too_wide {
        return Err(ErrorCode::OracleConfidenceTooWide.into())
    }

    // Returns true if the oracle price is stale for margin calculations
    let is_oracle_price_stale_for_margin = oracle_delay.gt(
        &oracle_validity_guard.slots_before_stale_for_margin
    );

    if is_oracle_price_stale_for_margin {
        return Err(ErrorCode::OraclePriceStaleForMargin.into())
    }

    // If else if else loop returning different outcomes of the OracleValidity enum based on different oracle
    // results
    let oracle_validity = if is_oracle_price_negative {
        OracleValidity::Invalid
    } else if is_oracle_price_too_volatile {
        OracleValidity::Volatile
    } else if is_confidence_too_wide {
        OracleValidity::Uncertain
    } else if is_oracle_price_stale_for_margin {
        OracleValidity::StaleForMargin
    } else if !has_sufficient_data_points {
        OracleValidity::InsufficientDataPoints
    } else {
        OracleValidity::Valid
    };

    Ok(oracle_validity)
}

/// Function to block an action(Action)
pub fn block_action(
    market: &Market,
    oracle_price_data: &OraclePriceData,
    guard_rails: &OracleGuardRails,
    last_acceptable_price: Option<u64>,
    historical_oracle_data: &HistoricalPriceData
) -> SpedXSpotResult<bool> {
    let OracleStatus {
        oracle_validity,
        is_mark_price_too_divergent,
        oracle_mark_spread_pct: _,
        ..
    } = get_oracle_status(
        oracle_price_data, 
        guard_rails, 
        historical_oracle_data, 
        last_acceptable_price
    )?;

    let is_oracle_valid = is_oracle_valid_for_action(
        Some(Actions::FillOrder),
        oracle_validity
    )?;

    let fills_paused = market.status == MarketStatus::FillsPaused;

    let block = !is_oracle_valid || is_mark_price_too_divergent || fills_paused;

    Ok(block)

    // Ok(true)
}

/// Struct monitoring the current oracle status.
/// Fields are the oracle price data, mark price divergence and oracle validity.
/// Acts as a wrapper for the oracle price and oracle validity data structures
#[derive(Default, Clone, Copy, Debug)]
pub struct OracleStatus {
    pub price_data: OraclePriceData,
    pub oracle_mark_spread_pct: i64,
    pub is_mark_price_too_divergent: bool,
    pub oracle_validity: OracleValidity
}

/// Function to fetch Oracle status with an explicit lifetime 'a
pub fn get_oracle_status<'a>(
    oracle_price_data: &'a OraclePriceData,
    guard_rails: &OracleGuardRails,
    historical_oracle_data: &HistoricalPriceData,
    last_acceptable_price: Option<u64>
) -> SpedXSpotResult<OracleStatus> {
    let oracle_validity = oracle_validity(
        historical_oracle_data.last_oracle_twap,
        oracle_price_data,
        &guard_rails.validity
    )?;

    let oracle_mark_spread_pct = calculate_oracle_twap_5min_mark_spread_pct(
        last_acceptable_price,
        historical_oracle_data
    )?;

    let is_oracle_mark_too_divergent = is_oracle_mark_too_divergent(
        oracle_mark_spread_pct, &guard_rails.price_divergence_guard_rails
    )?;

    Ok(
        OracleStatus {
            price_data: *oracle_price_data,
            oracle_mark_spread_pct,
            is_mark_price_too_divergent: is_oracle_mark_too_divergent,
            oracle_validity
        }
    )
}

/// Boolean function returning whether the mark price is too divergent from the oracle price. 
/// Maximum allowed divergence - 10% of the precision
pub fn is_oracle_mark_too_divergent(
    price_spread_pct: i64,
    oracle_mark_guard_rails: &PriceDivergenceGuardRails,
) -> SpedXSpotResult<bool> {
    let max_divergence = oracle_mark_guard_rails
        .oracle_mark_pct_divergence
        .max(PERCENTAGE_PRECISION_U64 / 10);

    Ok(price_spread_pct.unsigned_abs() > max_divergence)
}

/// Function to calculate the spread between the oracle 5min twap and the mark price in %.
pub fn calculate_oracle_twap_5min_mark_spread_pct(
    last_acceptable_price: Option<u64>,
    historical_oracle_data: &HistoricalPriceData
) -> SpedXSpotResult<i64> {
    let price = match last_acceptable_price {
        Some(price) => price,
        None => 0
    };

    let price_spread = price
        .cast::<i64>()?
        .safe_sub(historical_oracle_data.last_oracle_twap_5min)?;

    price_spread
        .cast::<i128>()?
        .safe_mul(BID_ASK_SPREAD_PRECISION_I128)?
        .safe_div(price.cast::<i128>()?)?
        .cast()
}

#[test]
fn calculate_oracle_validity() {
    // getting oracle price data
    let mut oracle_price_data = OraclePriceData {
        price: (34 * PRICE_PRECISION) as i64,
        confidence: PRICE_PRECISION_U64 / 100,
        delay: 1,
        has_sufficient_data_points: true,
    };

    // getting historical oracle data
    let mut historical_oracle_data = HistoricalPriceData {
        last_oracle_twap: (34 * PRICE_PRECISION) as i64,
        last_oracle_twap_time_stamp: 1656682258,
        last_oracle_twap_5min: (34 * PRICE_PRECISION) as i64,
        ..HistoricalPriceData::default()
    };

    // getting information of the guard rails for oracle and protocol validity
    let state = State {
        oracle_guard_rails: OracleGuardRails {
            price_divergence_guard_rails: PriceDivergenceGuardRails { 
                oracle_mark_pct_divergence: 1, 
                oracle_twap_5min_pct_divergence: 10 
            },
            validity: ValidityGuardRails { 
                slots_before_stale_for_margin: 120, 
                confidence_interval_max_accepted_divergence: 20000, 
                max_volatility_ratio: 5
            }
        },
        ..State::default()
    };

    // getting the oracle status at its current lifetime
    let mut oracle_status = get_oracle_status(&oracle_price_data, &state.oracle_guard_rails, &state.historical_price_data, None).unwrap();

    // we want to check that the oracle is valid and that the mark price is not too divergent
    assert!(oracle_status.oracle_validity == OracleValidity::Valid);
    assert!(!oracle_status.is_mark_price_too_divergent);

    // asserting that the oracle status is valid at different values of oracle price data
    oracle_price_data = OraclePriceData {
        price: (34 * PRICE_PRECISION) as i64,
        confidence: PRICE_PRECISION_U64 / 100,
        delay: 11,
        has_sufficient_data_points: true,
    };
    oracle_status = get_oracle_status(&oracle_price_data, &state.oracle_guard_rails, &state.historical_price_data, None).unwrap();
    assert!(oracle_status.oracle_validity != OracleValidity::Valid);

    // asserting that the oracle status is valid if there is a delay in oracle updates
    oracle_price_data.delay = 8;
    oracle_status = get_oracle_status(&oracle_price_data, &state.oracle_guard_rails, &state.historical_price_data, None).unwrap();
    assert_eq!(oracle_status.oracle_validity, OracleValidity::Valid);
    assert_eq!(!oracle_status.is_mark_price_too_divergent, false);

    // asserting that the oracle status is valid if there is a change in the 5min TWAP
    historical_oracle_data.last_oracle_twap_5min = 29*PRICE_PRECISION as i64;
    oracle_status = get_oracle_status(&oracle_price_data, &state.oracle_guard_rails, &state.historical_price_data, None).unwrap();
    assert_eq!(oracle_status.is_mark_price_too_divergent, false);
    assert_eq!(oracle_status.oracle_validity, OracleValidity::Valid);

    // asserting that the oracle status is valid with a certain confidence interval
    oracle_price_data.confidence = PRICE_PRECISION_U64;
    oracle_status = get_oracle_status(&oracle_price_data, &state.oracle_guard_rails, &state.historical_price_data, None).unwrap();
    assert_eq!(oracle_status.is_mark_price_too_divergent, false);
    assert_eq!(oracle_status.oracle_validity, OracleValidity::Volatile);

    // asserting that the oracle status is valid with pre-defined values of historical oracle data
    let price_precision = 32 * PRICE_PRECISION;
    let previous_ts = 1656682258;
    historical_oracle_data = HistoricalPriceData {
        last_oracle_twap: price_precision as i64,
        last_oracle_twap_time_stamp: previous_ts,
        ..HistoricalPriceData::default()
    };
    oracle_status = get_oracle_status(&oracle_price_data, &state.oracle_guard_rails, &historical_oracle_data, None).unwrap();
    assert_eq!(oracle_status.oracle_validity, OracleValidity::Valid);
    assert_eq!(!oracle_status.is_mark_price_too_divergent, false);
}
