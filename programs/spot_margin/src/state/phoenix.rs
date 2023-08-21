#![allow(unused_imports)]

use anchor_lang::prelude::*;
use crate::{
    state::oracle::*,
    math::constants::*, 
    math::safe_math::SafeMath,
    error::{
        SpedXSpotResult,
        ErrorCode
    },
    math::casting::Cast
};

use phoenix::{
    program::{
        new_order::{
            CondensedOrder,
            MultipleOrderPacket
        },
        CancelMultipleOrdersByIdParams,
        CancelOrderParams,
        // CancelOrWithdrawContext,
        MarketHeader,
    },
    quantities::WrapperU64,
    state::{
        markets::{
            FIFOMarket,
            FIFOOrderId,
            FIFORestingOrder,
            fifo, Market
        },
        OrderPacket,
        Side,
        SelfTradeBehavior
    }
};
use bytemuck::try_from_bytes;
use std::mem::size_of;

#[derive(Clone, Copy, Default)]
pub struct Phoenix;

impl Id for Phoenix {
    fn id() -> Pubkey {
        phoenix::id()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, Debug)]
pub struct PhoenixOrderIDs {
    pub price_in_ticks: u64,
    pub unique_order_num: u64,
}

pub fn fetch_market_header(account_info: &AccountInfo) -> SpedXSpotResult<MarketHeader> {
    assert_eq!(account_info.owner, &phoenix::id());
    let market = account_info.data.borrow();
    let market_header = try_from_bytes::<MarketHeader>(&market[..size_of::<MarketHeader>()]).map_err(|_| {
        msg!("Failed to deserialize phoenix market");
        ErrorCode::FailedToDeserializePhoenixMarket
    })?;

    assert_eq!(market_header.discriminant, PHOENIX_MARKET_DISCRIMINANT);

    Ok(*market_header)
}

pub fn get_price(
    base_price: u128,
    quote_price: u128,
    market_header: &MarketHeader
) -> u64 {
    let price = (base_price * (u64::pow(10, market_header.quote_params.decimals as u32) as u128)
        * (market_header.raw_base_units_per_base_unit as u128)
        / market_header.get_tick_size_in_quote_atoms_per_base_unit().as_u128() / quote_price) as u64;

    price
}