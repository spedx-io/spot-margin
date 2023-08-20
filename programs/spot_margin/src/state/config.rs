use anchor_lang::prelude::*;
use enumflags2::BitFlags;

use crate::{
    error::SpedXSpotResult,
    math::safe_unwrap::SafeUnwrap,
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

    /// Spot exchange status. We have state for exchange status, but here represent it in u8 bytes as 
    /// BitFlags returns the underlying value in bytes(u8 type)
    pub exchange_status: u8,

    /// Duration of a liquidation process
    pub liquidation_duration: u8,

    /// How much percentage of liabilities to liquidate
    pub initial_pct_liquidation: u16,

    pub padding: [u8;14],

}

impl State {
    /// Returns the exchange status by converting it from u8 bits to readable format
    pub fn get_exchange_status(&self) -> SpedXSpotResult<BitFlags<ExchangeStatus>> {
        BitFlags::<ExchangeStatus>::from_bits(usize::from(self.exchange_status)).safe_unwrap()
    }

    pub fn deposits_paused(&self) -> SpedXSpotResult<bool> {
        Ok(self.get_exchange_status()?.contains(ExchangeStatus::DepositsPaused))
    }

    pub fn withdraws_paused(&self) -> SpedXSpotResult<bool> {
        Ok(self.get_exchange_status()?.contains(ExchangeStatus::WithdrawsPaused))
    }

    pub fn fills_paused(&self) -> SpedXSpotResult<bool> {
        Ok(self.get_exchange_status()?.contains(ExchangeStatus::FillsPaused))
    }

    pub fn liquidations_paused(&self) -> SpedXSpotResult<bool> {
        Ok(self.get_exchange_status()?.contains(ExchangeStatus::LiquidationsPaused))
    }

    pub fn settlement_of_pnl_paused(&self) -> SpedXSpotResult<bool> {
        Ok(self.get_exchange_status()?.contains(ExchangeStatus::SettlementOfPnLPause))
    }
}

impl Size for State {
    const SIZE: usize = 992;
}