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
        }
    },
    state::{
        oracle::OraclePriceData,
        market::Market,
        traits::{
            SpotBalance,
            Size
        },
        enums::SpotBalanceType
    }
};

use anchor_lang::prelude::*;
use solana_program::msg;
use std::{
    cmp::max,
    panic::Location
};

#[zero_copy]
#[derive(Default, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Position {
    
}