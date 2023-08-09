//! Traits required for implementations
// #[cfg(test)]
// mod tests;

use crate::error::SpedXSpotResult;
use super::enums::SpotBalanceType;

/// Size of a market stored on-chain
pub trait Size { 
    const SIZE: usize;
}

/// Market index offset
pub trait MarketIndexOffset {
    const MARKET_INDEX_OFFSET: usize;   
}

pub trait SpotBalance {
    /// market for which balance should be fetched
    fn get_market_index(&self) -> u16;

    /// Balance type -- either deposits or borrows
    fn balance_type(&self) -> &SpotBalanceType;

    /// Balance available
    fn balance(&self) -> u128;

    /// Increase balance by either depositing or borrowing
    fn increase_balance(&mut self, delta: u128) -> SpedXSpotResult;

    /// Decrease balanace by either withdrawing or repaying
    fn decrease_balance(&mut self, delta: u128) -> SpedXSpotResult;

    /// Update balance type from deposits to borrows or vice versa
    fn update_balance_type(&mut self, balance_type: SpotBalanceType) -> SpedXSpotResult;
}

// pub trait SafeUnwrap {
//     type Item;

//     fn safe_unwrap(self) -> SpedXSpotResult<Self::Item>;
// }