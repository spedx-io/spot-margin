use anchor_lang::prelude::*;
use anchor_lang::ToAccountInfo;
use anchor_spl::token::{Token, TokenAccount};
use arrayref::array_ref;
use phoenix::{
    program::{
        create_new_order_instruction_with_custom_token_accounts, load_with_dispatch, MarketHeader
    },
    quantities::{
        BaseLots,
        QuoteLots,
        Ticks, 
        WrapperU64
    },
    state::{OrderPacket, Side},
};
use solana_program::{
    msg,
    program::invoke_signed_unchecked
};
use std::{
    cell::Ref,
    convert::TryInto,
    mem::size_of,
    ops::Deref
};
use crate::{
    load,
    state::enums::{PositionDirection, SpotFulfillmentType}
};