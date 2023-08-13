//! Function to calculate rolling sum of a defined TWAP.

use crate::{
    error::SpedXSpotResult,
    math::{
        casting::Cast,
        safe_math::SafeMath
    }
};
use std::cmp::max;

/// Logic to calculate rolling sum of a defined TWAP. We use 4 components for the calculation of the final rolling sum, the previous TWAP, the current price, the weightage numerator and the weightage denominator.
/// The previous TWAP represents the last available value of the TWAP, and is calculated using a price, a weightage numerator and denominator respectively.
/// A weightage numerator can represent x and weightage denominator can represent 100. This means that the previous TWAP would be calculated using the formula
/// (price*x)/100, and thus the weightage here would be x/100. After calculation of the previous TWAP, we add the curr_price to the previous TWAP to obtain the rolling sum.
pub fn calculate_rolling_sum(
    data_point_1: u64,
    data_point_2: u64,
    weightage_numerator: i64,
    weightage_denominator: i64
) -> SpedXSpotResult<u64> {
    let previous_twap = data_point_1
        .cast::<u128>()?
        .safe_mul(max(0, weightage_denominator.safe_sub(weightage_numerator)?).cast::<u128>()?)?
        .safe_div(weightage_denominator.cast::<u128>()?)?;

    previous_twap.cast::<u64>()?.safe_add(data_point_2)
}