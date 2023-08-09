use anchor_lang::prelude::*;

use crate::math::constants::PERCENTAGE_PRECISION_U64;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct OracleGuardRails {
    pub price_divergence_guard_rails: PriceDivergenceGuardRails,

    pub validity: ValidityGuardRails,
}

impl Default for OracleGuardRails {
    fn default() -> Self {
        OracleGuardRails {
            price_divergence_guard_rails: PriceDivergenceGuardRails::default(),
            validity: ValidityGuardRails {
                slots_before_stale_for_margin: 120, // 60 seconds,
                confidence_interval_max_accepted_divergence: 20_000, // 2% of price,
                max_volatility_ratio: 5, // 5x
            }
        }
    }
}

impl OracleGuardRails {
    pub fn max_oracle_twap_5min_oracle_pct_divergence(&self) -> u64 {
        self.price_divergence_guard_rails
            .oracle_twap_5min_pct_divergence
            .max(PERCENTAGE_PRECISION_U64/2)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct PriceDivergenceGuardRails {
    /// Divergence between the oracle price and the mark price in %.
    pub oracle_mark_pct_divergence: u64,

    /// Max divergence of the 5-min oracle TWAP in %. 
    pub oracle_twap_5min_pct_divergence: u64,
}

impl Default for PriceDivergenceGuardRails {
    fn default() -> Self {
        PriceDivergenceGuardRails {
            oracle_mark_pct_divergence: PERCENTAGE_PRECISION_U64/10, // 10%
            oracle_twap_5min_pct_divergence: PERCENTAGE_PRECISION_U64/2, // 50%
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct ValidityGuardRails {
    /// The number of slots after which the price is considered stale for margining.
    pub slots_before_stale_for_margin: i64,

    /// The maxiumum accepted width of confidence intervals
    pub confidence_interval_max_accepted_divergence: u64,

    /// The maximum accepted volatility 
    pub max_volatility_ratio: i64
}
