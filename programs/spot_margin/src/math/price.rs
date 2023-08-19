//! Math utilities for price-related functions
use crate::{
    state::{
        enums::PositionDirection,
        // order::Order,
    }, 
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