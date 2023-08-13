//! Function to calculate TWAP(Time-weighted average price)

use crate::{
    error::SpedXSpotResult,
    math::{
        casting::Cast,
        safe_math::SafeMath
    }
};
use std::cmp::max;

/// Logic to calculate weighted average using 2 data points and 2 weightages.
/// Each data point is assigned a weightage by multiplying the data point with a weightage value. 
/// For example, a data point representing a price of $100 with a weightage of .1 will be assigned a value of 10.
/// If any of the weightage is 0, we return the other data point, whose assigned weightage !=0. 
/// We also have checks in place in case if a weightage >1(weightages should never be >1). If any weightage exceeds 1,
/// we apply a discount. We use 3 components for the calculation of the final TWAP, the weighted price, the last TWAP and a denominator.
/// The weighted price represents the price to which a weight has been applied. The last TWAP represents the previous TWAP value, which also
/// is a price which has been weighted. The denominator is the sum of the weightages. The formula for calculating the TWAP is
/// (last_twap + curr_weighted_price)/weightage_1+weightage_2
pub fn weighted_average(
    data_point_1: i64,
    data_point_2: i64,
    weightage_1: i64,
    weightage_2: i64
) -> SpedXSpotResult<i64> {
    let denominator = weightage_1.safe_add(weightage_2)?.cast::<i128>()?;

    let previous_twap = data_point_1.cast::<i128>()?.safe_mul(weightage_1.cast()?)?;

    let weighted_price = data_point_2.cast::<i128>()?.safe_mul(weightage_2.cast()?)?;

    if weightage_1 == 0 {
        return Ok(data_point_2);
    }

    if weightage_2 == 0 {
        return Ok(data_point_1);
    }

    let x = if weightage_2 > 1 {
        if weighted_price < previous_twap {
            -1
        } else if weighted_price > previous_twap {
            1
        } else {
            0
        }
    } else {
        0
    };

    let twap = previous_twap
        .safe_add(weighted_price)?
        .safe_div(denominator)?
        .cast::<i64>()?;

    twap.safe_add(x)
}

/// Calculating a TWAP using the `weighted_average` fn and parameters for the curr_price representing the weighted_price,
/// last_twap representing the last TWAP, backwards representing the weightage_1 and forwards representing the weightage_2.
/// Because we are calculating the TWAP, the weights naturally are calculated using time periods. Hence, the weights backwards
/// and forwards are time periods too. The backwards weightage is calculated by subtracting the current timestamp from the last
/// timestamp of the TWAP. The forwards weightage is calculated by subtracting the period for which the TWAP is calculated from
/// the backwards weightage. The formula for calculating the TWAP is (curr_price + last_twap)/(backwards+forwards)
pub fn calculate_twap(
    curr_price: i64,
    curr_ts: i64,
    last_twap: i64,
    last_ts_of_twap: i64,
    period_for_twap_calc: i64
) -> SpedXSpotResult<i64> {
    let backwards = max(0_i64, curr_ts.safe_sub(last_ts_of_twap)?);

    let forwards = max(1_i64, period_for_twap_calc.safe_sub(backwards)?);

    weighted_average(curr_price, last_twap, backwards, forwards)
}