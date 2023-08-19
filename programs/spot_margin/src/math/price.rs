//! Math utilities for price-related functions
use crate::{
    state::enums::PositionDirection, 
    error::{SpedXSpotResult, ErrorCode},
};

use super::safe_math::SafeMath;

pub fn standardize_price(
    price: u64,
    tick_size: u64,
    position_direction: PositionDirection,
) -> SpedXSpotResult<u64> {
    if price == 0 {
        return Ok(0)
    }

    let remainder = price.checked_rem_euclid(tick_size).ok_or_else(|| ErrorCode::MathError)?;

    if remainder == 0 {
        return Ok(price)
    }

    match position_direction {
        PositionDirection::Long => price.safe_sub(remainder),
        PositionDirection::Short => price.safe_add(tick_size.safe_sub(remainder)?),
        PositionDirection::TwoWay => price.safe_add(tick_size)
    }
}

pub fn standardize_price_i64(
    price: i64,
    tick_size: i64,
    position_direction: PositionDirection
) -> SpedXSpotResult<i64> {
    if price == 0 {
        return Ok(0)
    }

    let remainder = price.checked_rem_euclid(tick_size).ok_or_else(|| ErrorCode::MathError)?;

    if remainder == 0 {
        return Ok(price)
    }

    match position_direction {
        PositionDirection::Long => price.safe_sub(remainder),
        PositionDirection::Short => price.safe_add(tick_size.safe_sub(remainder)?),
        PositionDirection::TwoWay => price.safe_add(tick_size)
    }
}

pub fn standardize_base_asset_amt(
    base_asset_amt: u64,
    order_step_size: u64
) -> SpedXSpotResult<u64> {
    let remainder = base_asset_amt
        .checked_rem_euclid(order_step_size)
        .ok_or_else(|| ErrorCode::MathError)?;

    base_asset_amt.safe_sub(remainder)
}

pub fn standardize_base_asset_amt_ceil(
    base_asset_amt: u64,
    order_step_size: u64
) -> SpedXSpotResult<u64> {
    let remainder = base_asset_amt
        .checked_rem_euclid(order_step_size)
        .ok_or_else(|| ErrorCode::MathError)?;

    if remainder == 0 {
        Ok(base_asset_amt)
    } else {
        base_asset_amt.safe_add(order_step_size)?.safe_sub(remainder)
    }
}

pub fn is_base_asset_amt_multiple_of_order_step_size(
    base_asset_amt: u64,
    order_step_size: u64
) -> SpedXSpotResult<bool> {
    let remainder = base_asset_amt
        .checked_rem_euclid(order_step_size)
        .ok_or_else(|| ErrorCode::MathError)?;

    Ok(remainder == 0)
}

