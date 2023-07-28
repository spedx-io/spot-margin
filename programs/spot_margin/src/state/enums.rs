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