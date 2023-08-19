use anchor_lang::zero_copy;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::{SpedXSpotResult, ErrorCode};
use crate::math::{
    casting::Cast,
    safe_math::SafeMath,
    price::standardize_price
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
    /// The slot in which the order was placed
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

    // pub market_type: MarketType

    /// Client-generated order ID. This field can be used to place orders/block orders + cancel orders/block orders
    pub user_order_id: u8,

    /// What the user's side/direction was when an order was placed. 
    /// If TwoWay is selected, simultaneously an order is placed on the opposite side of the book
    pub existing_position_direction: PositionDirection,

    /// Whether the user want's to go long or short when opening a new position.
    /// If TwoWay is selected, simultaneously an order is placed on the opposite side of the book
    pub new_position_direction: PositionDirection,

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
                standardize_price(limit_price.cast::<u64>()?, tick_size, self.new_position_direction)?
            )
        } else if self.price == 0 {
            match fallback_price {
                Some(price) => Some(standardize_price(price, tick_size, self.new_position_direction)?),
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
}