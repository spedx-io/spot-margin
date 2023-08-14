//! Functions for representing a user's position in spot markets.

use crate::{
    error::{
        SpedXSpotResult,
        ErrorCode
    },
    math::{
        casting::Cast,
        constants::{
            EPOCH_DURATION,
            OPEN_ORDER_MARGIN_REQUIREMENT,
            PRICE_PRECISION_I128,
            QUOTE_PRECISION_U64,
            QUOTE_PRECISION,
            QUOTE_SPOT_MARKET_INDEX,
            THIRTY_DAY
        },
        safe_math::SafeMath,
        balance::{
            get_amount_signed,
            get_token_value,
            get_amount_of_tokens
        },
        rolling_sum::calculate_rolling_sum, oracle_validity
    },
    state::{
        oracle::OraclePriceData,
        market::Market,
        traits::{
            SpotBalance,
            Size
        },
        enums::{
            SpotBalanceType,
            PositionDirection
        }
    },
    safe_increment,
    update_struct_id,

};
use anchor_lang::prelude::*;
use solana_program::msg;
use std::{
    cmp::max,
    panic::Location
};

// use super::enums::PositionDirection;

/// Represents a user's position in a corresponding spot market. Contains details such as 
/// - user balance
/// - base asset value
/// - quote asset value
/// - quote asset breakeven amount
/// - open asks and open bids active in the corresponding spot market
/// - any settled pnl against the market
#[zero_copy]
#[derive(Default, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Position {
    /// Scaled balance of the user account. Obtained using the formula: balance*(cumulative_deposit/borrow_interest_for_market)
    /// precision: SPOT_BALANCE_PRECISION
    pub scaled_balance: u64,

    /// The size of the user's position. Represents the number of base tokens the user has bought in a position.
    /// precision: BASE_PRECISION
    pub base_asset_amount: i64,

    /// The amount used to value the base asset for a position. Calculated using (base_asset_amount*avg_entry_price)-applicable_fees
    /// precision: QUOTE_PRECISION
    pub quote_asset_amount: i64,

    /// Represents the amount of quote asset receivable by a user after exiting a position in order to breakeven. 
    /// Breakeven here represents a state of neutrality, neither profit nor loss. Thus this field represents how much of the quote asset
    /// that the user must receive in order to recoup his losses from the same position.
    /// precision: QUOTE_PRECISION
    pub quote_break_even: i64,

    /// In simpler terms, represents base_asset_amount+fees(-+ fees are cancelled so as to obtain quote asset amount excluding fees)
    /// precision: QUOTE_PRECISION
    pub quote_asset_amount_without_fees: i64,

    /// The amount of PnL settled(not realized) in a market since a position ha sbeen opened.
    /// precision: QUOTE_PRECISION(as PnL is settled in the quote asset)
    pub settled_pnl: i64,

    /// Represents the number of open bid orders the user has for a market
    /// precision: token mint
    pub open_bids: i64,

    /// Represents the number of open ask orders the user has for a market
    /// precision: token mint
    pub open_asks: i64,

    /// The cumulative deposits that a user has made into a market. Can also represent the cumulative borrows a user has taken 
    /// from a borrow lending market.
    pub cumulative_deposits_for_market: i64,

    /// The market index of the corresponding spot market
    pub market_index: u16,

    /// Whether the position(action) is of deposit or borrow nature.
    pub bal_type: SpotBalanceType,

    /// Number of open orders that a user has
    pub num_open_orders: u8,

    /// Whether two way orders are enabled or not. 
    /// Two way orders are orders that are placed on both sides of the orderbook.
    pub two_way_orders_enabled: bool,

    pub padding: [u8;4],
}

impl SpotBalance for Position {
    fn get_market_index(&self) -> u16 {
        self.market_index
    }

    fn balance_type(&self) -> &SpotBalanceType {
        &self.bal_type // need to add & as we are referencing spot balance type
    }

    fn balance(&self) -> u128 {
        self.scaled_balance as u128
    }

    fn increase_balance(&mut self, delta: u128) -> SpedXSpotResult {
        self.scaled_balance = self.scaled_balance.safe_add(delta.cast()?)?;
        Ok(())
    }

    fn decrease_balance(&mut self, delta: u128) -> SpedXSpotResult {
        self.scaled_balance = self.scaled_balance.safe_sub(delta.cast()?)?;
        Ok(())
    }

    fn update_balance_type(&mut self, balance_type: SpotBalanceType) -> SpedXSpotResult {
        self.bal_type = balance_type;
        Ok(())
    }
}

impl Position {
    pub fn is_empty(&self) -> bool {
        self.scaled_balance == 0 && self.num_open_orders == 0
    }

    pub fn has_open_orders(&self) -> bool {
        self.num_open_orders > 0 || self.open_bids > 0 || self.open_asks > 0
    }

    pub fn has_open_position(&self) -> bool {
        self.base_asset_amount != 0
    }

    pub fn has_settled_pnl(&self) -> bool {
        self.base_asset_amount == 0 && self.quote_asset_amount >0
    }

    pub fn are_two_way_orders_enabled(&self) -> bool {
        self.two_way_orders_enabled == true
    }

    /// CHECK: multiply the number of orders to the minimum margin requirement to obtain cumulative margin requirement to
    /// open a position
    pub fn margin_requirement(&self) -> SpedXSpotResult<u128> {
        self.num_open_orders.cast::<u128>()?.safe_mul(OPEN_ORDER_MARGIN_REQUIREMENT)
    }

    pub fn get_token_amount(&self, market: &Market) -> SpedXSpotResult<u128> {
        get_amount_of_tokens(self.scaled_balance.cast()?, market, &self.bal_type) // &self.bal_type is a reference to the enum
    }

    pub fn get_token_amount_signed(&self, market: &Market) -> SpedXSpotResult<i128> {
        get_amount_signed(
            get_amount_of_tokens(self.scaled_balance.cast()?, market, &self.bal_type)?,
            &self.bal_type
        )
    }

    pub fn get_token_amount_unstrict(
        &self,
        market: &Market,
        oracle_price_data: &OraclePriceData,
        twap_5min: Option<i64>,
        amount: Option<i128>
    ) -> SpedXSpotResult<[i128;2]> {
        let amount = match amount {
            Some(amount) => amount,
            None => self.get_token_amount_signed(market)?,
        };

        let amount_after_all_bids_get_filled = amount.safe_add(self.open_bids.cast()?)?;

        let amount_after_all_asks_get_filled = amount.safe_add(self.open_asks.cast()?)?;

        let oracle_price = match twap_5min {
            Some(twap_5min) => twap_5min.max(oracle_price_data.price),
            None => oracle_price_data.price
        };

        if amount_after_all_bids_get_filled.abs() > amount_after_all_asks_get_filled.abs() {
            let order_value_unstrict = get_token_value(-self.open_bids as i128, market.decimals, oracle_price)?;
            Ok([amount_after_all_bids_get_filled, order_value_unstrict])
        } else {
            let order_value_unstrict = get_token_value(-self.open_asks as i128, market.decimals, oracle_price)?;
            Ok([amount_after_all_asks_get_filled, order_value_unstrict])
        }
    }

    // pub fn get_position_direction(&self) -> PositionDirection {
    //     if self.base_asset_amount > 0 {
    //         PositionDirection::Long
    //     } else if self.base_asset_amount < 0 {
    //         PositionDirection::Short
    //     } else {
    //         PositionDirection::TwoWay
    //     }
    // }

    // pub fn get_direction_to_close_position(&self) -> PositionDirection {
    //     if self.base_asset_amount >0 {
    //         PositionDirection::Short
    //     } else if self.base_asset_amount < 0 {
    //         PositionDirection::Long
    //     } else {
    //         PositionDirection::TwoWay
    //     }
    // }

    pub fn get_position_direction(&self) -> PositionDirection {
        if self.two_way_orders_enabled == true {
            if self.base_asset_amount > 0 {
                PositionDirection::Long
            } else if self.base_asset_amount < 0 {
                PositionDirection::Short
            } else {
                PositionDirection::TwoWay
            }
        } else {
            if self.base_asset_amount > 0 {
                PositionDirection::Long
            } else {
                PositionDirection::Short
            }
        }
    }

    pub fn get_opposite_direction(&self) -> PositionDirection {
        if self.two_way_orders_enabled == true {
            if self.base_asset_amount > 0 {
                PositionDirection::Short
            } else if self.base_asset_amount < 0 {
                PositionDirection::Long
            } else {
                PositionDirection::TwoWay
            }
        } else {
            if self.base_asset_amount > 0 {
                PositionDirection::Short
            } else {
                PositionDirection::Long
            }
        }
    }

    pub fn get_position_cost_schedule(&self) -> SpedXSpotResult<i128> {
        if self.base_asset_amount == 0 {
            return Ok(0);
        }

        (-self.quote_asset_amount.cast::<i128>()?)
            .safe_mul(PRICE_PRECISION_I128)?
            .safe_div(self.base_asset_amount.cast()?)
    }

}