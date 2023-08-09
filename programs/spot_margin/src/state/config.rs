use anchor_lang::prelude::*;
use enumflags2::BitFlags;

use crate::{
    error::SpedXSpotResult,
    math::{
        constants::{
            FEE_DENOMINATOR,
            FEE_PERCENTAGE_DENOMINATOR,
            PERCENTAGE_PRECISION_U64
        },
        safe_unwrap::SafeUnwrap
    },
    state::{
        traits::Size,
        enums::ExchangeStatus,
        guard_rails::OracleGuardRails
    }
};

/// Global state of the protocol
#[derive(Default)]
#[repr(C)]
#[account]
pub struct State {
    /// Market admin pubkey
    pub admin: Pubkey,

    /// Pubkey of token that can be minted
    pub whitelist_mint: Pubkey,
    
    /// Pubkey of token that does not provide guarantee of minting
    pub discount_mint: Pubkey,

    /// Pubkey of the signer of the instruction
    pub signer: Pubkey,

    // pub spot_fee_structure: FeeStructure,

    pub oracle_guard_rails: OracleGuardRails,

    /// Number of authorities for a market
    pub number_of_authorities: u64,

    /// Cooldown for providing liquidity
    pub lp_cooldown_time: u64,

    /// We have a buffer rate added on maintenance margin. So the user must have maintenance margin 
    /// + buffer rate to avoid liquidation
    pub liquidation_margin_buffer_ratio: u32,

    /// Duration until settlement of a trade happens
    pub settlement_duration: u16,

    // pub number_of_markets: u16,

    /// Number of spot markets available
    pub number_of_spot_markets: u16,

    /// Signer nonce
    pub signer_nonce: u8,

    /// Time-in-force configuration of a market order
    pub default_market_order_time_in_force: u8,

    /// Spot exchange status
    pub exchange_status: u8,

    /// Duration of a liquidation process
    pub liquidation_duration: u8,

    /// How much percentage of liabilities to liquidate
    pub initial_pct_liquidation: u16,

    pub padding: [u8;14],

}

impl State {}

impl Size for State {
    const SIZE: usize = 992;
}