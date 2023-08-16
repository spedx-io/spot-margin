// use core::num::bignum::Big32x40;
use std::fmt::{
    Display,
    Formatter
};
use enumflags2::BitFlags;

use borsh::{BorshDeserialize, BorshSerialize};

// use super::traits::Size

/// Enums for Position Directions(Longs and Shorts)
#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, PartialEq, Debug, Eq)]
pub enum PositionDirection {
    Long,
    Short,
    TwoWay
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
            PositionDirection::TwoWay => PositionDirection::TwoWay
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

/// Enum returning the current status of SpedX
#[derive(BitFlags, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExchangeStatus {
    DepositsPaused = 0b00000001,
    WithdrawsPaused = 0b00000010,
    FillsPaused = 0b00000100,
    LiquidationsPaused = 0b00001000,
    SettlementOfPnLPause = 0b00010000
}

impl ExchangeStatus {
    /// Returns an ExchangeStatus type by creating an empty BitFlags value and returning the underlying value
    /// in u8 bytes 
    pub fn active() -> u8 {
        BitFlags::<ExchangeStatus>::empty().bits() as u8
    }
}

/// Different outcomes of oracle validity. Ranging from descending order of severity or invalidity.
/// Invalid represents that the extreme state of invalidity whereas Valid represents the extreme state of validity.
#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, PartialEq, Debug, Eq)]
pub enum OracleValidity {
    Invalid,
    Volatile,
    Uncertain,
    StaleForMargin,
    InsufficientDataPoints,
    Valid,
}

impl Default for OracleValidity {
    fn default() -> Self {
        OracleValidity::Valid
    }
}

/// Enum representing different actions(both client and server-side) on the protocol
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq)]
pub enum Actions {
    PnLSettlement,
    OrderAdded,
    FillOrder,
    Liquidate,
    MarginCalculation,
    UpdateTWAP,
}

/// Enum representing the current user status, with descending order of severity
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Liquidatable,
    BeingLiquidated,
    Bankrupt,
    ReduceOnly,
    Passive
}

impl Default for UserStatus {
    fn default() -> Self {
        UserStatus::Active
    }
}

impl super::traits::Size for UserStatus {
    const SIZE: usize = 4376;
}

/// The asset class we would be referring to in a specific context, either the base or quote asset of a market
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum AssetClass {
    BaseAsset,
    QuoteAsset
}

/// Current status of an order
#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, PartialEq, Eq, Debug)]
pub enum OrderStatus {
    NotInitialized,
    Active,
    Filled,
    Cancelled
}

/// Different order types
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug)]
pub enum OrderType {
    ImmediateOrCancel,
    Limit,
    PostOnly,
    TriggerMarket, // Stop Loss, Take Profit
    TriggerLimit, // Stop Loss Limit, Take Profit Limit
    OraclePegged
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::Limit
    }
}

/// Returns whether a Trigger should be triggered above(take-profit) or below(stop-loss) a certain price. 
/// If it's already triggered, return TriggeredAbove for orders triggered above the limit price, and TriggeredBelow for orders triggered below the limit price
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug)]
pub enum OrderTriggerConditions {
    Above,
    Below,
    TriggeredAbove,
    TriggeredBelow
}

/// Default value for OrderTriggerConditions is Above(take-profit)
impl Default for OrderTriggerConditions {
    fn default() -> Self {
        OrderTriggerConditions::Above
    }
}