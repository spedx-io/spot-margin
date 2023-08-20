use std::cmp::max;

// use borsh::{
//     BorshSerialize,
//     BorshDeserialize
// };
// use solana_program::msg;

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
        oracle::{OraclePriceData, HistoricalPriceData},
        enums::{
            MarketStatus,
            OracleValidity,
            Actions
        },
        guard_rails::{
            OracleGuardRails,
            ValidityGuardRails,
            PriceDivergenceGuardRails
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
        ema: _oracle_ema
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
            oracle_mark_spread_pct: oracle_mark_spread_pct,
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