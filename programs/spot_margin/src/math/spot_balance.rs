//! Program-level code to determine balance in account of user

use solana_program::msg;

use crate::{
    error::{
        SpedXSpotResult,
        ErrorCode
    },
    math::{
        casting::Cast,
        constants::{
            ONE_YEAR,
            SPOT_RATE_PRECISION,
            SPOT_UTILIZATION_PRECISION
        },
        safe_math::{
            SafeFloorDiv,
            SafeMath
        },
    },
    state::oracle::OraclePriceData
};

pub fn calculate_utilization(
    deposit_token_account: u128,
    borrow_token_account: u128
) -> SpedXSpotResult<u128> {
    // utilization formula, borrows multiplied by precision divided by the deposits
    let utilization = borrow_token_account
        .safe_mul(SPOT_UTILIZATION_PRECISION)?
        .checked_div(deposit_token_account)
        .unwrap_or({
            if deposit_token_account == 0 && borrow_token_account == 0 {
                // there are no borrows nor deposits
                0_128
            } else {
                // if there are borrows without any deposits, default to maximum utilization
                SPOT_UTILIZATION_PRECISION
            }
        });
    
    Ok(utilization)
}