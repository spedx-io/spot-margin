use std::fmt::{
    Display,
    Formatter
};

use borsh::{BorshDeserialize, BorshSerialize};

/// Enums for Position Directions(Longs and Shorts)
#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, PartialEq, Debug, Eq)]
pub enum PositionDirection {
    Long,
    Short,
}

/// Default position for PositionDirection
impl Default for PositionDirection {
    /// Default PositionDirection is long. 
    fn default() -> Self {
        PositionDirection::Long
    }
}

impl PositionDirection {
    /// Opposite side of the current position. Returns a PositionDirection type
    pub fn opposite(&self) -> Self {
        match self {
            PositionDirection::Long => PositionDirection::Short,
            PositionDirection::Short => PositionDirection::Long,
        }
    }
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq)]
pub enum SpotFulfillmentType {
    OpenBookV2,
    PhoenixV1
}

impl Default for SpotFulfillmentType {
    fn default() -> Self {
        SpotFulfillmentType::PhoenixV1
    }
}

/// Oracle types
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq)]
pub enum OracleType {
    Pyth,
    Switchboard,
    QuoteAsset,
    Pyth1K,
    Pyth1M,
    PythStables
}

impl Default for OracleType {
    fn default() -> Self {
        OracleType::Pyth
    }
}

/// Balance of a user account can either be in Deposits or Borrows
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug)]
pub enum SpotBalanceType {
    /// Balance can be in the form of deposits, i.e deposited by the user from his account
    Deposits,
    /// Balance can also be in the form of borrowed capital
    Borrows
}

impl Display for SpotBalanceType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SpotBalanceType::Borrows => write!(f,"SpotBalanceType::Borrows"),
            SpotBalanceType::Deposits => write!(f,"SpotBalanceType::Deposits"),
        }
    }
}

impl Default for SpotBalanceType {
    fn default() -> Self {
        SpotBalanceType::Deposits
    }
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug)]
pub enum MarketStatus {
    /// Period succeeding market initialization, fills are paused
    Initialized,
    
    /// All operations are allowed
    Active,

    // FundingPaused,

    /// Fills are paused, so order's can't be fulfilled in this time period
    FillsPaused,

    /// For spot market, refers to pausing of depositing of an asset, i.e withdrawal from wallet
    WithdrawPaused,

    /// Fills are only able to reduce the size
    ReduceOnly,

    /// Market is in settlement mode. Market has determined settlement price, and expired positions must be settled
    Settlement,

    /// Market has no participants, and is delisted
    Delisted
}

impl Default for MarketStatus {
    fn default() -> Self {
        MarketStatus::Initialized
    }
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum AssetTier {
    /// Free collateral, can be used for borrows
    Collateral,

    /// Can be used as collateral for positions, but cannot be borrowed against.
    /// Can also be thought as collateral not eligible for borrowing.
    Protected,

    /// Collateral locked up in cross margin positions
    CrossMargined,

    /// Collateral locked up in isolated margin positions
    IsolatedMargined,

    /// Collateral not supported
    Unlisted
}

impl Default for AssetTier {
    fn default() -> Self {
        AssetTier::Unlisted
    }
}