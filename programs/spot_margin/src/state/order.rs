use anchor_lang::zero_copy;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::{SpedXSpotResult, ErrorCode};
use crate::math::{
    casting::Cast,
    safe_math::SafeMath,
    price::{
        standardize_price,
        standardize_base_asset_amt
    }
};

use super::enums::{
    PositionDirection,
    OrderType,
    OrderStatus,
    OrderTriggerConditions
};
use solana_program::msg;
use std::panic::Location; 

#[zero_copy]
#[repr(C)]
#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug, Eq)]
pub struct Order {
    pub slot: u64,

    /// The price at which a (Limit) order was placed. Price is not relevant for Market orders. 
    /// For ImmediateOrCancel(IOC) orders, if the order matches fewer lots than a lots threshold, the order is cancelled.
    /// For FillOrKill(FOK) orders, which is a subset of FOK orders, the order size, or the number of base/quote lots must always be equal to the lots threshold.
    /// precision: PRICE_PRECISION
    pub price: u64,

    /// Represents the base lots of the order, or in simple terms, the size of the order.
    /// precision: token mint precision
    pub base_asset_amount: u64,

    /// Represents the part of the initial order size that has been filled. For an order size of 1BTC, if only 0.8BTC has been filled, this field returns that amount.
    /// precision: token mint precision
    pub base_asset_filled: u64,

    /// Represents the amount of quote lots filled for the order. 
    /// For an order size of $30,000, if only $24,000 has been filled, this field returns that amount.
    /// precision: QUOTE_PRECISION
    pub quote_asset_filled: u64,

    /// The price at which the order will be triggered. Only valid for trigger orders such as SL, SM, TP, TPL.
    /// precision: PRICE_PRECISION
    pub price_for_trigger_orders: u64,

    /// Represents a time-period for which the order shall stay active, after which it expires. 
    /// A value of 0 for this field indicates that this order never expires.
    /// Time-in-force orders can be of precisely two typeS: slot and a unix timestamp.
    /// The order will be voided after the the specifided slot and the same shall be true if the curr_ts eclipses the unix_ts.
    /// precision: TIME_IN_FORCE_PRECISION 
    pub time_in_force: i64,

    /// A field inspired from Ellipsis Labs' Phoenix Spot DEX, which silently cancel's an order if there are insufficient funds for the order to execute.
    pub fail_silently_on_insufficient_funds_error: bool,

    /// When this field has a value, the order limit price is equal to the oracle price + the value of this field.
    /// Hence, this acts as a spread between the order limit price and the oracle price.
    /// precision: PRICE_PRECISION
    pub oracle_limit_spread: i32,

    /// Unique order ID. On the blockchain, each user is allocated sufficient space for their order IDs.
    pub order_id: u32,

    /// Order type. Currently supported order types are:
    /// - ImmediateOrCancel[Includes Market, IOC + FOK orders]
    /// - Limit
    /// - PostOnly
    /// - TriggerOrders
    /// - OraclePegged
    pub order_type: OrderType,

    /// Unique market index
    pub market_index: u16,

    /// Current status of an order. 
    /// Can be NotInitialized, Active(Open), Filled and Cancelled
    pub order_status: OrderStatus,

    /// Client-generated order ID. This field can be used to place orders/block orders + cancel orders/block orders
    pub user_order_id: u8,

    /// What the user's side/direction was when an order was placed. 
    /// If TwoWay is selected, simultaneously an order is placed on the opposite side of the book
    pub existing_position_direction: PositionDirection,

    /// Whether the user wants to long or short.
    /// If TwoWay is selected, simultaneously an order is placed on the opposite side of the book
    pub pos_direction: PositionDirection,

    /// Whether an incoming new order is allowed to reduce size of an existing position. 
    /// Allows for reducing exposure for positions. TwoWay are NEVER reduce-only. As their main intention is to profit off of
    /// market movements on both the sides. Reduce-only does not allow for this, as although exposure of market movements against one side is reduced
    /// by simply reducing the size of the position, and not placing an order on the other side of the book.
    pub reduce_only: bool,

    /// If set to true, the order will always be a maker. PostOnly orders are 
    pub post_only: bool,

    /// If set to true, the order will be cancelled if it cannot be filled upto min_base_lots/min_quote_lots, i.e of the type ImmediateOrCancel
    pub immediate_or_cancel: bool,
    
    /// If set to true, the order will be cancelled if it is not completely filled immediately
    /// i.e num_base_lots = min_base_lots || num_quote_lots = min_quote_lots. 
    pub fill_or_kill: bool,

    /// Returns to true if the order is executed without regard to the price, i.e at the best available price.
    pub treat_ioc_as_market: bool,

    /// Whether the order is triggered above or below the set limit price. Only relevant for Trigger order types.
    pub trigger_conditions: OrderTriggerConditions,

    pub padding: [u8;3],
}

impl Default for Order {
    fn default() -> Self {
        Order {
            order_status: OrderStatus::NotInitialized,
            order_type: OrderType::Limit,
            slot: 0,
            order_id: 0,
            user_order_id: 0,
            price: 0,
            existing_position_direction: PositionDirection::Long,
            base_asset_amount: 0,
            base_asset_filled: 0,
            quote_asset_filled: 0,
            pos_direction: PositionDirection::Long,
            reduce_only: false,
            post_only: false,
            immediate_or_cancel: false,
            price_for_trigger_orders: 0,
            trigger_conditions: OrderTriggerConditions::Above,
            oracle_limit_spread: 0,
            fail_silently_on_insufficient_funds_error: true,
            treat_ioc_as_market: true,
            fill_or_kill: false,
            time_in_force: 0,
            market_index: 0,
            padding: [0;3]
        }
    }
}

impl Order {
    pub fn does_order_have_oracle_price_offset(&self) -> bool {
        self.oracle_limit_spread !=0
    }

    pub fn get_limit_price(
        &self,
        last_acceptable_oracle_price: Option<i64>,
        fallback_price: Option<u64>,
        tick_size: u64
    ) -> SpedXSpotResult<Option<u64>> {
        let price = if self.does_order_have_oracle_price_offset() {
            let oracle_price = last_acceptable_oracle_price.ok_or_else(|| {
                msg!("Oracle not found, hence unable to calculate spread");
                ErrorCode::OracleNotFound
            })?;

            let limit_price = oracle_price.safe_add(self.oracle_limit_spread.cast()?)?;

            if limit_price <= 0 {
                msg!("Limit price cannot be equal to or lesser than zero: {}", limit_price);
                return Err(ErrorCode::InvalidOracleSpreadLimitPrice)
            }

            Some(
                standardize_price(limit_price.cast::<u64>()?, tick_size, self.pos_direction)?
            )
        } else if self.price == 0 {
            match fallback_price {
                Some(price) => Some(standardize_price(price, tick_size, self.pos_direction)?),
                None => None
            }
        } else {
            Some(self.price)
        };

        Ok(price)
    }

    pub fn force_get_limit_price(
        &self,
        last_acceptable_oracle_price: Option<i64>, 
        fallback_price: Option<u64>,
        tick_size: u64
    ) -> SpedXSpotResult<u64> {
        match self.get_limit_price(last_acceptable_oracle_price, fallback_price, tick_size)? {
            Some(limit_price) => Ok(limit_price),
            None => {
                let caller = Location::caller();
                msg!(
                    "Error while fetching limit price at {}:{}",
                    caller.file(),
                    caller.line()
                );
                Err(ErrorCode::UnableToGetLimitPrice)
            }
        }
    }

    pub fn does_order_have_limit_price(
        &self,
    ) -> SpedXSpotResult<bool> {
        Ok(self.price > 0 ||
            self.does_order_have_oracle_price_offset())
    }

    pub fn get_unfilled_base_amount(
        &self,
        existing_position: Option<i64>,
        num_lots_to_fill: u64,
    ) -> SpedXSpotResult<u64> {

        let unfilled_base_amt = self.base_asset_amount.safe_sub(self.base_asset_filled)?;

        let existing_position = match existing_position { 
            Some(existing_position) => existing_position,
            None => {
                return Ok(unfilled_base_amt)
            }
        };

        if !self.reduce_only || self.post_only {
            return Ok(unfilled_base_amt);
        }

        if existing_position == 0 {
            return Ok(0)
        }

        if self.fill_or_kill {
            return Ok(0)
        }

        if self.immediate_or_cancel {
            if self.base_asset_filled < num_lots_to_fill {
                return Err(ErrorCode::IOCOrderCancelledDueToNumBaseLotsNotFilled);
            } else if self.base_asset_filled > num_lots_to_fill {
                return Ok(num_lots_to_fill)
            } else {
                return Ok(num_lots_to_fill)
            }
        }

        match self.pos_direction {
            PositionDirection::Long => {
                if existing_position > 0 {
                    Ok(0)
                } else {
                    Ok(unfilled_base_amt.min(existing_position.unsigned_abs()))
                }
            },
            PositionDirection::Short => {
                if existing_position < 0 {
                    Ok(0)
                } else {
                    Ok(unfilled_base_amt.min(existing_position.unsigned_abs()))
                }
            },
            PositionDirection::TwoWay => {
                if existing_position > 0 {
                    Ok(unfilled_base_amt.min(existing_position.unsigned_abs()))
                } else if existing_position < 0 {
                    Ok(unfilled_base_amt.min(existing_position.unsigned_abs()))
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub fn get_standardized_base_amount_unfilled(
        &self,
        existing_position: Option<i64>,
        num_base_lots_to_fill: u64,
        order_step_size: u64,
    ) -> SpedXSpotResult<u64> {
        standardize_base_asset_amt(
            self.get_unfilled_base_amount(existing_position, num_base_lots_to_fill)?,
            order_step_size
        )
    }

    pub fn order_has_trigger(&self) -> bool {
        matches!(
            self.order_type,
            OrderType::TriggerMarket | OrderType::TriggerLimit
        )
    }

    pub fn has_trigger_order_triggered(&self) -> bool {
        matches!(
            self.trigger_conditions,
            OrderTriggerConditions::TriggeredAbove | OrderTriggerConditions::TriggeredBelow
        )
    }

    pub fn order_has_trigger_above(&self) -> bool {
        matches!(
            self.trigger_conditions,
            OrderTriggerConditions::Above | OrderTriggerConditions::TriggeredAbove
        )
    }

    pub fn order_has_trigger_below(&self) -> bool {
        matches!(
            self.trigger_conditions,
            OrderTriggerConditions::Below | OrderTriggerConditions::TriggeredBelow
        )
    }

    pub fn order_has_triggered_above(&self) -> bool {
        matches!(
            self.trigger_conditions,
            OrderTriggerConditions::TriggeredAbove
        )
    }

    pub fn order_has_triggered_below(&self) -> bool {
        matches!(
            self.trigger_conditions,
            OrderTriggerConditions::TriggeredBelow
        )
    }

    pub fn is_order_open(&self, market_index: u16) -> bool {
        self.market_index == market_index && self.order_status == OrderStatus::Active
    }

    pub fn is_order_ioc(&self, min_num_lots_to_fill: u64) -> bool {
        self.base_asset_filled >= min_num_lots_to_fill
    }

    pub fn is_order_fok(&self, min_num_lots_to_fill: u64) -> bool {
        self.base_asset_amount == min_num_lots_to_fill
    }

    pub fn is_market_order(&self) -> bool {
        matches!(
            self.order_type,
            OrderType::TriggerMarket | OrderType::ImmediateOrCancel | OrderType::OraclePegged
        ) && self.immediate_or_cancel
    }

    pub fn is_limit_order(&self) -> bool {
        matches!(
            self.order_type,
            OrderType::Limit | OrderType::TriggerLimit
        )
    }

    pub fn is_resting_limit_order(&self) -> bool {
        self.is_limit_order() && self.post_only && !self.fill_or_kill && !self.immediate_or_cancel
    }
}