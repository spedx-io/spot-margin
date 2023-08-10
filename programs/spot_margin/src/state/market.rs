use std::{
    fmt,
    fmt::{
        Display,
        Formatter
    }, ffi::NulError,
};

use anchor_lang::prelude::*;

use crate::{
    error::{
        SpedXSpotResult,
        ErrorCode
    },
    math::{
        casting::Cast,
        constants::{
            MARGIN_PRECISION,
            SPOT_BALANCE_PRECISION_U64,
            PRICE_PRECISION_I64,
            SPOT_CUMULATIVE_INTEREST_PRECISION
        },
        safe_math::SafeMath,
        spot_balance::calculate_utilization
    },
    state::{
        oracle::{
            HistoricalIndexData,
            HistoricalPriceData,
        },
        enums::{
            OracleType,
            MarketStatus,
            AssetTier
        },
        traits::{
            Size,
            MarketIndexOffset,
            SpotBalance
        }
    },
    validate
};


#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
#[account(zero_copy)]
pub struct Market {
    /// Address of the market. PDA of the market index
    pub pubkey: Pubkey,

    /// The oracle used for pricing(Pyth and switchboard for now)
    pub oracle: Pubkey,

    /// Token mint of the market
    pub token_mint: Pubkey,

    /// The vault used to store the market's deposits.
    /// By default, borrows are facilitated through this vault, and thus vault value must be greater than total deposits
    /// minus total borrows
    pub vault: Pubkey,

    /// Name of the market in bytes. 
    pub name: [u8; 32],

    /// Oracle data for historical prices
    pub historical_oracle_data: HistoricalPriceData,

    /// Oracle data for historical prices of the index asset
    pub historical_index_data: HistoricalIndexData,

    /// Revenue(includes fees and other forms of collectibles) the protocol has generated(in the market's quote currency)
    /// Most of our markets will be denominated in USDC, with the exception of a few denominated in LSTs.
    pub revenue_pool: PoolBalance,

    /// Represents the total amount of fees collected by the protocol from swaps between markets.
    /// Ultimately, it is settled to the market's revenue pool. 
    pub spot_fee_pool: PoolBalance,

    /// The market's PnL pool. 
    pub pnl_pool: PoolBalance,

    // pub insurance_fund: InsuranceFund,

    /// Total fees collected for this market. Precision: QUOTE_PRECISION, as fees are collected in quote asset.
    /// precision: QUOTE_PRECISION
    pub total_spot_fee: u128,

    /// Sum of scaled balances across user accounts and pool balances(revenue and fee pool).
    /// The balance might not indicate the true amount including interest, hence we multiply by the cumulative deposit interest
    /// To convert it to the actual amount.
    /// precision: SPOT_BALANCE_PRECISION
    pub deposit_balance: u128,

    /// Sum of scaled balances across user accounts and pool balances(revenue and fee pool).
    /// The balance might not indicate the true amount including interest, hence we multiply by the cumulative borrow interest
    /// To convert it to the true amount.
    /// precision: SPOT_BALANCE_PRECISION
    pub borrow_balance: u128,

    /// The interest earned by depositors into the platform
    /// Used to calculate the deposit balance for users and pool balance.
    /// precision: SPOT_CUMULATIVE_INTEREST_PRECISION
    pub cumulative_deposit_interest: u128,

    /// The interest earned by borrowers from the platform
    /// Used to calculate the borrow balance for users and pool balance.
    /// precision: SPOT_CUMULATIVE_INTEREST_PRECISION
    pub cumulative_borrow_interest: u128,

    /// The total amount of socialized loss charged from traders to cover up for losses occured on borrows.
    /// Paid up in the mint's token
    pub cumulative_socialized_loss: u128,

    /// The total amount of socialized loss charged from traders to cover up for losses occured on borrows.
    /// Paid up in the market's quote currency.
    /// precision: QUOTE_PRECISION
    pub cumulative_quote_socialized_loss: u128,

    /// Threshold to protect the protocol from a sudden drain in liquidity.
    /// When initial deposits made by the user are below the said threshold, no limits will be enforced.
    pub withdraw_limit: u64,

    /// The maximum amount of deposits that are intakeable for this market.
    /// We would not want excess deposits that then can be used for borrowing excess collateral at a high LTV.
    /// precision: token mint
    pub max_deposit_limit: u64,

    /// 24hr TWAP of token deposits
    /// precision: token mint
    pub token_deposit_twap: u64,

    /// 24hr TWAP of token borrows
    /// precision: token mint
    pub token_borrow_twap: u64,

    /// 24hr TWAP of utilization ratios(borrow amount/deposit amount)
    /// precision: SPOT_UTILIZATION_PRECISION
    pub utilization_twap: u64,

    /// Record of the last available deposits and borrow inteerst rates,
    pub last_interest_ts: u64,

    /// Last time the deposit, borrow and utilization twaps were updated
    pub last_twap_ts: u64,

    /// Remaining time until the market expires. Only applicable when the market is in reducw-only mode
    pub expiry_ts: i64,

    /// Orders against a market must be a multiple of a given step size.
    /// precision: mint precision
    pub order_step_size: u64,

    /// Orders against a market must be a multiple of a given tick size.
    /// precision: PRICE_PRECISION
    pub order_tick_size: u64,

    /// Minimum order size
    /// precision: mint precision
    pub min_order_size: u64,

    /// Maximum allowed position size.
    /// We would want to safeguard the market from excess concentration of whales
    /// precision: mint precision
    pub max_position_size: u64,

    /// Every trade that has occured has a fill. Hence, every trade has a fill record id, representing the fill.
    /// This field represents the id for the next fill record
    pub next_fill_record_id: u64,

    /// Every deposit that has occured has an id. This field represents the id for the next deposit record
    pub next_deposit_record_id: u64,

    /// The protocol uses asset weights to determine the share of deposits to a user's initial collateral.
    /// For an asset weight of 80%, 80% of the user's deposits contribute to initial collateral.
    /// precision: SPOT_WEIGHT_PRECISION
    pub initial_asset_weight: u32,

    /// The protocol uses asset weights to determine the share of deposits to a user's maintenance collateral.
    /// For an asset weight of 80%, 80% of the user's deposits contribute to maintenance collateral.
    /// precision: SPOT_WEIGHT_PRECISION
    pub maintenance_asset_weight: u32,

    /// The protocol uses liability weights to determine the share of borrows to a user's initial collateral.
    /// For an liability weight of 80%, 80% of the user's borrows contribute to initial collateral.
    /// precision: SPOT_WEIGHT_PRECISION. 
    /// Note that using borrows to open a margin trading position further increases borrows atop the initial borrow.
    pub initial_liability_weight: u32,

    /// The protocol uses liability weights to determine the share of borrows to a user's maintenance collateral.
    /// For an liability weight of 80%, 80% of the user's borrows contribute to maintenance collateral.
    /// precision: SPOT_WEIGHT_PRECISION.
    /// Note that using borrows to open a margin trading position further increases borrows atop the initial borrow.
    pub maintenance_liability_weight: u32,

    /// The protocol uses an IMF(initial margin factor), inspired from FTX(no comments)'s risk engine.
    /// This basically ensures that initial asset weights of large positions are discounted.
    /// precision: MARGIN_PRECISION
    pub imf_factor: u32,

    /// The fee that the liquidator is paid for taking over borrows
    /// precision: LIQUIDATOR_FEE_PRECISION
    pub liquidator_fee: u32,

    /// The fee that the insurance fund is paid for taking over borrows
    /// precision: LIQUIDATOR_FEE_PRECISION
    pub insurance_fund_liquidation_fee: u32,

    /// Optimal utilization ratio for this market. Optimal utilization is used to determine the borrow rate.
    /// precision: SPOT_UTILIZATION_PRECISION
    pub optimal_utilization: u32,

    /// Optimal borrow rate is a function of optimal utilization.
    /// precision: SPOT_RATE_PRECISION
    pub optimal_borrow_rate: u32,

    /// Maximum borrow rate one can incur at 100% utilization.
    pub max_borrow_rate: u32,

    /// The market's token mint's decimals. precision: 10^decimals
    pub decimals: u32,

    /// Unsigned-16bit integer type representing the unique index of the market
    pub market_index: u16,

    /// Whether the market is currently accepting orders
    pub orders_enabled: bool,

    /// Oracle type being used for the market, to price liquidations
    pub oracle_type: OracleType,

    /// Current state of the market, whether it is initialized, active, etc.
    pub status: MarketStatus,

    /// AssetTier enum dictates how a deposit(collateral) is treated, as well as how liquidations on different types of margin
    /// are handled.
    pub asset_tier: AssetTier,

    pub padding1: [u8;6],

    /// The amount loaned out in the begin_swap instruction
    /// precision: token mint
    pub flash_loan_amount: u64,

    /// The amount in the users token account in the begin_swap instruction
    /// Used to calculate how much of the token left with the system in the end_swap instruction
    /// precision: token mint
    pub flash_loan_initial_token_amount: u64,

    /// Fees received from swaps
    /// precision: token mint
    pub total_swap_fee: u64,

    /// The maximum pnl imbalance before unrealized positive pnl asset weights are discounted
    /// PnL imbalance refers to the skew of longs PnL over shorts or vice versa. The protocol typically
    /// and stochastically aims for 0 imbalance, i.e the protocol and it's traders are market neutral. 
    /// precision: QUOTE_PRECISION
    pub unrealized_pnl_max_imbalance: u64,

    // pub insurance_claim: InsuranceClaim,

    /// The price at which the positions will be settled if and when the market expires
    /// precision: PRICE_PRECISION
    pub expiry_price: i64,

    /// The imf factor for unrealized PnL. Used to discount asset weight for large positive PnL 
    /// precision: MARGIN_PRECISION
    pub unrealized_pnl_imf: u32,

    /// The minimum margin ratio which is required to open a position.
    /// At 0.1, 10% of the position value must be collateralized.
    /// precision: MARGIN_PRECISION
    pub min_margin_ratio_initial: u32,

    /// The minimum margin ratio which is required to maintain a position.
    /// At 0.05, 5% of the total collateral must be held in the account.
    /// If the account balance dips below this ratio, they are open for liquidation
    pub min_margin_ratio_maintenance: u32,

    /// The initial asset weights for Unrealized PnL. 
    /// precision: SPOT_WEIGHT_PRECISION
    pub unrealized_pnl_initial_asset_weight: u32,

    /// The maintenance asset weights for Unrealized PnL.
    pub unrealized_pnl_maintenance_asset_weight: u32,

    /// Number of users having an open position
    pub number_of_users: u32,

    // pub contract_tier: ContractTier,
    
    pub padding: [u8; 56],
}

#[derive(Default, Eq, PartialEq, Debug)]
#[repr(C)]
#[zero_copy]
pub struct PoolBalance {
    /// To accurately obtain the pool's token amount, we multiply the balance with the market's deposit interest precision.
    pub scaled_balance: u128,

    /// The market that the pool belongs to. Each market has its own revenue pool
    pub market_index: u16,

    pub padding: [u8; 6],
}

impl SpotBalance for PoolBalance {
    fn get_market_index(&self) -> u16 {
        self.market_index
    }

    fn balance_type(&self) -> &super::enums::SpotBalanceType {
        &super::enums::SpotBalanceType::Deposits
    }

    fn balance(&self) -> u128 {
        self.scaled_balance
    }

    fn increase_balance(&mut self, delta: u128) -> SpedXSpotResult {
        self.scaled_balance += delta;
        Ok(())
    }

    fn decrease_balance(&mut self, delta: u128) -> SpedXSpotResult {
        self.scaled_balance -= delta;
        Ok(())
    }

    fn update_balance_type(&mut self, _balance_type: super::enums::SpotBalanceType) -> SpedXSpotResult {
        Err(ErrorCode::CannotUpdatePoolBalanceType.into())
    }
}

impl Default for Market {
    fn default() -> Self {
        Market {
            pubkey: Pubkey::default(), // market's pubkey,
            oracle: Pubkey::default(), // oracle's pubkey,
            token_mint: Pubkey::default(), // token mint's pubkey,
            vault: Pubkey::default(), // vault's pubkey,
            name: [0;32],
            historical_oracle_data: HistoricalPriceData::default(),
            historical_index_data: HistoricalIndexData::default(),
            revenue_pool: PoolBalance::default(),
            spot_fee_pool: PoolBalance::default(),
            total_spot_fee: 0,
            deposit_balance: 0,
            borrow_balance: 0,
            cumulative_deposit_interest: 0,
            cumulative_borrow_interest: 0,
            cumulative_socialized_loss: 0,
            cumulative_quote_socialized_loss: 0,
            withdraw_limit: 0,
            max_deposit_limit: 0,
            token_deposit_twap: 0,
            token_borrow_twap: 0,
            utilization_twap: 0,
            last_interest_ts: 0,
            last_twap_ts: 0,
            expiry_ts: 0,
            order_step_size: 1, // orders must be a multiple of this step size, i.e 1
            order_tick_size: 0,
            min_order_size: 0,
            max_position_size: 0,
            next_fill_record_id: 0,
            next_deposit_record_id: 0,
            initial_asset_weight: 0,
            initial_liability_weight: 0,
            maintenance_asset_weight: 0,
            maintenance_liability_weight: 0,
            imf_factor: 0,
            liquidator_fee: 0,
            insurance_fund_liquidation_fee: 0,
            optimal_utilization: 0,
            optimal_borrow_rate: 0,
            max_borrow_rate: 0,
            decimals: 0,
            market_index: 0,
            orders_enabled: false, // orders disabled by default, otherwise traders rekt with 0 values
            oracle_type: OracleType::default(), // pyth,
            status: MarketStatus::default(),
            asset_tier: AssetTier::default(),
            padding1: [0;6],
            flash_loan_amount: 0,
            flash_loan_initial_token_amount: 0,
            total_swap_fee: 0,
            padding: [0;56],
            pnl_pool: PoolBalance::default(),
            unrealized_pnl_max_imbalance: 0,
            expiry_price: 0,
            unrealized_pnl_imf: 0,
            min_margin_ratio_initial: 0,
            min_margin_ratio_maintenance: 0,
            unrealized_pnl_initial_asset_weight: 0,
            unrealized_pnl_maintenance_asset_weight: 0,
            number_of_users: 0
        }
    }
}

/// Pre-allocated size of spot market
impl Size for Market {
    const SIZE: usize = 776;
}

/// Offset for market index
impl MarketIndexOffset for Market {
    const MARKET_INDEX_OFFSET: usize = 684;
}

impl Market {
    pub fn is_market_active(&self, now: i64) -> SpedXSpotResult<bool> {

        // market is considered active if it is not in settlement mode or delisted
        let market_status_active = !matches!(
            self.status,
            MarketStatus::Settlement | MarketStatus::Delisted
        );

        // market is considered as not expired if the expiry timestamp is 0
        // or if the current timestamp is less than the expiry timestamp, meaning that there is still time left
        // for the market to expire
        let market_not_expired = self.expiry_ts == 0 || now < self.expiry_ts;
        Ok(market_status_active && market_not_expired)
    }

    /// Returns true if the market is in reduce only mode
    pub fn market_in_reduce_only_mode(&self) -> bool {
        self.status == MarketStatus::ReduceOnly
    }

    /// Returns true if market is in active, reduce only, withdraws paused and not in fills paused mode.
    pub fn are_fills_enabled(&self) -> bool {
        matches!(
            self.status,
            MarketStatus::Active | MarketStatus::ReduceOnly | MarketStatus::WithdrawPaused
        );
        !matches!(
            self.status,
            MarketStatus::FillsPaused
        )
    }

}