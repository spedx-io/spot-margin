use std::cmp::max;

use borsh::{
    BorshSerialize,
    BorshDeserialize
};
use solana_program::msg;

use crate::{
    error::{
        SpedXSpotResult,
        ErrorCode
    },
    math::{
        casting::Cast,
        constants::BID_ASK_SPREAD_PRECISION,
        safe_math::SafeMath,
    },
    state::{
        oracle::OraclePriceData,
        enums::{
            MarketStatus,
            OracleValidity,
            Actions
        },
        guard_rails::{
            OracleGuardRails,
            ValidityGuardRails
        }
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

    let is_oracle_price_stale_for_margin = oracle_delay.gt(
        &oracle_validity_guard.slots_before_stale_for_margin
    );

    if is_oracle_price_stale_for_margin {
        return Err(ErrorCode::OraclePriceStaleForMargin.into())
    }

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