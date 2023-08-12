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
    state::{
        oracle::OraclePriceData,
        market::Market,
        enums::SpotBalanceType
    }, 
    validate
};

pub struct InterestAccumulated {
    pub deposits_interest: u128,
    pub borrows_interest: u128
}

/// Function to get balance in spot assets of a user
pub fn get_spot_asset_balance(
    amount: u128,
    market: &Market,
    balance_type: &SpotBalanceType,
    approximate_up: bool
) -> SpedXSpotResult<u128> {
    // increasing the precision by adding an exponent of 19-market_decimals to the base of 10
    let precision_uptick = 10_u128.pow(19_u32.safe_sub(market.decimals)?);

    // Calculating the cumulative interest receivable for deposits or payable for borrows
    let cumulative_interest_deposit_borrow = match balance_type {
        SpotBalanceType::Deposits => market.cumulative_deposit_interest,
        SpotBalanceType::Borrows => market.cumulative_borrow_interest
    };

    // calculating balance by applying the cumulative deposit/borrow interest
    let mut balance = amount.safe_mul(precision_uptick)?.safe_div(cumulative_interest_deposit_borrow)?;

    // if approximation is set to true, round up the balance
    if approximate_up && balance>0 {
        balance = balance.safe_add(1)?;
    }

    Ok(balance)
}

/// Function to get amount of tokens including any receivables(assets) and/or payables(liabilities) in precision downticks
pub fn get_amount_of_tokens(
    balance: u128,
    market: &Market,
    balance_type: &SpotBalanceType
) -> SpedXSpotResult<u128> {
    // decreasing the precision by adding an exponent of 19-market_decimals to the base of 10
    let precision_downtick = 10_u128.pow(19_u32.safe_sub(market.decimals)?);

    let cumulative_interest_deposit_borrow = match balance_type {
        SpotBalanceType::Deposits => market.cumulative_deposit_interest,
        SpotBalanceType::Borrows => market.cumulative_borrow_interest
    };

    let amount = match balance_type {
        SpotBalanceType::Deposits => {
            balance.safe_mul(cumulative_interest_deposit_borrow)?
            .safe_div(precision_downtick)?
        },
        SpotBalanceType::Borrows => {
            balance.safe_mul(cumulative_interest_deposit_borrow)?
            .safe_ceil_div(precision_downtick)?
        }
    };

    Ok(amount)
}

/// Function returning the amount of tokens receivable or payable in signed form, i.e signs applied.
/// Returns a i128 value accounting for -ve amounts for borrows. +ve amounts for deposits are cast.
pub fn get_amount_signed(
    amount: u128,
    balance_type: &SpotBalanceType
) -> SpedXSpotResult<i128> {
    match balance_type {
        SpotBalanceType::Deposits => amount.cast(),
        // For borrows, return -ve amount for a variable called amount of type signed 128 bytes
        SpotBalanceType::Borrows => amount.cast::<i128>().map(|amount| -amount)
    }
}

/// Function to get interest amount in precision downticks
pub fn get_interest(
    balance: u128,
    market: &Market,
    interest: u128
) -> SpedXSpotResult<u128> {
    let precision_downtick = 10_u128.pow(19_u32.safe_sub(market.decimals)?);

    let interest_amount = balance.safe_mul(interest)?.safe_div(precision_downtick)?;

    Ok(interest_amount)
}

/// Function to calculate utilization of the borrow lending pool
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

pub fn calculate_per_market_utilization(market: &Market) -> SpedXSpotResult<u128> {
    // get amount of deposits for a spot market
    let deposits = get_amount_of_tokens(
        market.deposit_balance, 
        market, 
        &SpotBalanceType::Deposits
    )?;

    // get amount of borrows for a spot market
    let borrows = get_amount_of_tokens(
        market.borrow_balance, 
        market, 
        &SpotBalanceType::Borrows
    )?;

    // calculating utilization using deposit token account and borrow token account for the market
    let utilization = calculate_utilization(
        deposits, 
        borrows
    )?;

    Ok(utilization)
}

/// Function to calculate total accumulated interest in the structure of `InterestAccumulated` referecing
/// how much total accumulated interest is receivable or payable for deposits and borrows respectively.
pub fn calculate_accumulated_interest(
    market: &Market,
    now_ts: i64
) -> SpedXSpotResult<InterestAccumulated> {
    // Calculating utilization for a market. Meaning that we want to calculate interest accumulated from margin trading
    // against a market
    let utilization = calculate_per_market_utilization(market)?;


    // if utilization is zero, return 0 for both deposit and borrow interest. As interest is calculated 
    if utilization == 0 {
        return Ok(
            InterestAccumulated {
                deposits_interest: 0,
                borrows_interest: 0
            }
        )
    }

    // however, if utilization > 0 && utilization > optimal utilization, calculate the surplus utilization.
    let borrow_rate = if utilization > market.optimal_utilization.cast()? {
        let surplus = utilization.safe_sub(market.optimal_utilization.cast()?)?;
    
        // calculating the slope of the borrow rate curve. calculated using the formula:
        // (max_borrow_rate-optimal_borrow_rate)*utilization_precision/(utilization_precision-optimal_utilization)
        let borrow_rate_slope = market
            .max_borrow_rate
            .cast::<u128>()?
            .safe_sub(market.optimal_borrow_rate.cast()?)?
            .safe_mul(SPOT_UTILIZATION_PRECISION)?
            .safe_div(
                SPOT_UTILIZATION_PRECISION
                    .safe_sub(market.optimal_utilization.cast()?)?,
            )?;
        
        // after calculating the slope, we calculate the borrow rate(in case of surplus utilization) using the formula:
        // (optimal_borrow_rate + (surplus*slope/utilization_precision)). We divide by utilization_precision to
        // cancel out the multiplication of utilization_precision in the borrow_rate_slope calculation. So, the end result would be
        // (optimal_borrow_rate + (surplus_utilization*slope))
        market.optimal_borrow_rate.cast::<u128>()?.safe_add(
            surplus
                .safe_mul(borrow_rate_slope)?
                .safe_div(SPOT_UTILIZATION_PRECISION)?
        )?
    } else {
        // else, if the utilization < optimal_utilization, we tweak the formula to
        // (optimal_borrow_rate*utilizarion_precision/optimal_utilization).
        let borrow_rate_slope = market
             .optimal_borrow_rate
            .cast::<u128>()?
            .safe_mul(SPOT_UTILIZATION_PRECISION)?
            .safe_div(market.optimal_utilization.cast()?)?;
    
        // after calculating the slope, we calculate the utilization(in case of deficit utilization) using the formula:
        // (utilization*slope/utilization_precision). We divide by utilization_precision to
        // cancel out the multiplication of utilization_precision in the borrow_rate_slope calculation. So, the end result would be
        // (utilization*slope)

        utilization
            .safe_mul(borrow_rate_slope)?
            .safe_div(SPOT_UTILIZATION_PRECISION)?

        // Some notes with regards to borrow rates, utilization and borrow rate slopes:
        // -> Interest rates in general are calculated based on utilization. so, higher utilization -> higher rates
        // -> the typical formula of a borrow rate curve is optimal_borrow_rate/optimal_utilization. 
        // -> Meaning that it is calculated from the graph of borrow rate and utilization. 
        // -> In order to prevent calculation of borrow rate slope using highly skewed values, we only take into account
        //    optimal borrow rates and optimal utilization.

        // -> However, in the scenarios of utilization > optimal utilization, the formula is tweaked and the numerator
        // -> is now represented by the difference of the maximum attainable borrow rate for a market and its optimal borrow rate.
        // -> We use the maximum attainable borrow rate, accounting for the fact that the curr_utilization > optimal_utilization
        //    and thus the curr borrow rate would naturally be higher than the optimal borrow rate.
        // -> After obtaining the difference between the max borrow rate and optimal borrrow rate, we divide it by the difference
        // -> between the utilization precision and optimal utilization. This is to account for the fact the current utilization is 
        // -> higher than the optimal utilization and we are using a smaller value(as it is a difference) in the numerator
    };

    // calculating the time since the interest was last updated. formula: now_timestamp - last_interest_timestamp
    let time_since_last_update_of_interest = now_ts
        .cast::<u64>()
        .or(Err(ErrorCode::UnableToCastUnixTimestamp))?
        .safe_sub(market.last_interest_ts)?;

    // This operation costs higher execution costs. Hence, we are having to multiply with `time_since_last_update_of_interest`
    // and then dividing by ONE_YEAR constant to even out during the calculation of the interest accumulation
    let new_borrow_rate = borrow_rate.safe_mul(time_since_last_update_of_interest as u128)?;

    // calculating new deposit rate using the formula: 
    // (new_borrow_rate*utilization)/utilization_precision. We divide by utilization_precision to cancel out the multiplication
    // of utilization_precision in the borrow_rate_slope calculation. in case of deficit calculation, the new_deposit_ratio would
    // evaluate to utilization^2*slope. In case of surplus calculation, the new_deposit_ratio would evaluate to
    // optimal_borrow_rate + utilization*surplus*slope
    let new_deposit_rate = new_borrow_rate
        .safe_mul(utilization)?
        .safe_div(SPOT_UTILIZATION_PRECISION)?;

    // calculating cumulative borrow interest using the formula:
    // -> (cumulative_borrow_interest*new_borrow_rate/ONE_YEAR*rate_precision) + 1. We divide by ONE_YEAR to cancel out the multiplication
    // -> of `time_since_last_update_of_interest` in the new_borrow_rate calculation. We divide by rate_precision as all metrics of the protocol
    // -> such as interest rates, spread etc are in precise form only. We add 1 to account for credit spread.
    let borrow_interest = market
        .cumulative_borrow_interest
        .safe_mul(new_borrow_rate)?
        .safe_div(ONE_YEAR)?
        .safe_div(SPOT_RATE_PRECISION)?
        .safe_add(1)?;

    // Similar process are calculating of borrow_interest, instead, we do not add 1 as we want to maintain spread between
    // the deposit and borrow rates
    let deposit_interest = market
        .cumulative_deposit_interest
        .safe_mul(new_deposit_rate)?
        .safe_div(ONE_YEAR)?
        .safe_div(SPOT_RATE_PRECISION)?;

    // returning the interest accumulated
    Ok(
        InterestAccumulated {
            deposits_interest: deposit_interest,
            borrows_interest: borrow_interest
        }
    )
}

/// Function to get value of a given amount of tokens using oracle pricing.
/// The lower value of the oracle pricing and its twap is used to price that amount.
pub fn get_strict_value(
    amount: i128,
    decimals: u32,
    oracle_price_data: &OraclePriceData,
    oracle_price_twap: i64
) -> SpedXSpotResult<i128> {
    if amount == 0 {
        return Ok(0)
    }

    let precision_downtick = 10i128.pow(decimals);

    validate!(
        oracle_price_twap > 0 && oracle_price_data.price > 0,
        ErrorCode::InvalidOracle,
        "oracle price data = {:?} && oracle price twap = {}",
        oracle_price_data,
        oracle_price_twap
    )?;

    let price = if amount > 0 {
        if oracle_price_data.price < oracle_price_twap {
            oracle_price_data.price
        } else {
            oracle_price_twap
        }
    } else {
        if oracle_price_data.price > oracle_price_twap {
            oracle_price_data.price
        } else {
            oracle_price_twap
        }
    };

    let token_value = amount.safe_mul(price.cast()?)?;

    if token_value < 0 {
        token_value.safe_floor_div(precision_downtick)
    } else {
        token_value.safe_div(precision_downtick)
    }
}

/// Function to get value of a given amount of tokens using oracle pricing.
/// Only the oracle pricing is used to price the amount
pub fn get_token_value(
    amount: i128,
    decimals: u32,
    oracle_price: i64
) -> SpedXSpotResult<i128> {
    if amount == 0 {
        return Ok(0);
    }

    let precision_downtick = 10i128.pow(decimals);

    let token_value_using_oracle = amount.safe_mul(oracle_price.cast()?)?;

    if token_value_using_oracle < 0 {
        token_value_using_oracle.safe_floor_div(precision_downtick.abs())
    } else {
        token_value_using_oracle.safe_div(precision_downtick)
    }
}