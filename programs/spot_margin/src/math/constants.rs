use solana_program::native_token::LAMPORTS_PER_SOL; // exponent = 9
pub const LAMPORTS_PER_SOL_U64: u64 = LAMPORTS_PER_SOL;
pub const LAMPORTS_PER_SOL_I64: i64 = LAMPORTS_PER_SOL as i64;

// spot market constants
pub const QUOTE_SPOT_MARKET_INDEX: u16 = 0;

// user spot market constants
pub const MAX_SPOT_POSITIONS: u8 = 8;
pub const MAX_OPEN_ORDERS: u8 = 32;

// precision for base asset i.e the number of decimal places supported for representing the base asset
pub const BASE_PRECISION: u128 = 1_000_000_000; // exponent = -9
pub const BASE_PRECISION_I128: i128 = BASE_PRECISION as i128;
pub const BASE_PRECISION_U64: u64 = BASE_PRECISION as u64;
pub const BASE_PRECISION_I64: i64 = BASE_PRECISION_I128 as i64;

// price precisions i.e the number of decimal places supported for a price 
pub const PRICE_PRECISION: u128 = 1_000_000; // exponent = -6
pub const PRICE_PRECISION_I128: i128 = PRICE_PRECISION as i128;
pub const PRICE_PRECISION_U64: u64 = PRICE_PRECISION as u64;
pub const PRICE_PRECISION_I64: i64 = PRICE_PRECISION_I128 as i64;

// precision for quote basset i.e the number of decimal places supported for representing the quote asset
pub const QUOTE_PRECISION: u128 = 1_000_000; // exponent = -6
pub const QUOTE_PRECISION_I128: i128 = QUOTE_PRECISION as i128;
pub const QUOTE_PRECISION_U64: u64 = QUOTE_PRECISION as u64;
pub const QUOTE_PRECISION_I64: i64 = QUOTE_PRECISION_I128 as i64;

// margin precision i.e the number of decimal places supported for representing the initial and maintenance margin
pub const MARGIN_PRECISION: u32 = 10_000; 
pub const MARGIN_PRECISION_U128: u128 = MARGIN_PRECISION as u128;

// spot weights basically refers to the % of deposited asset can be used as collateral for margin trading 
pub const SPOT_WEIGHT_PRECISION: u32 = MARGIN_PRECISION;
pub const SPOT_WEIGHT_PRECISION_U128: u128 = SPOT_WEIGHT_PRECISION as u128;

// Refers to precision for liquidation rewards
pub const LIQUIDATION_PCT: u128 = 10_000;

// Precision for balance of spot assets availabe
pub const SPOT_BALANCE_PRECISION: u128 = 1_000_000_000; // exponent = -9
pub const SPOT_BALANCE_PRECISION_U64: u64 = SPOT_BALANCE_PRECISION as u64;

// Refers to the precision of interest earned/paid in spot assets. Interested earned => lending, Interest paid => borrows
pub const SPOT_CUMULATIVE_INTEREST_PRECISION: u128 = 10_000_000_000; // exponent = -10

// percentage and basis point precisions
pub const PERCENTAGE_PRECISION: u128 = 1_000_000; // expo -6 (represents 100%)
pub const PERCENTAGE_PRECISION_I128: i128 = PERCENTAGE_PRECISION as i128;
pub const PERCENTAGE_PRECISION_U64: u64 = PERCENTAGE_PRECISION as u64;
pub const TEN_BPS: i128 = PERCENTAGE_PRECISION_I128 / 1000;
pub const TEN_BPS_I64: i64 = TEN_BPS as i64;
pub const TWO_PT_TWO_PCT: i128 = 22_000;

// Bid Ask spread precisions, i.e the number of decimal places supported for representing the bid ask spread
pub const BID_ASK_SPREAD_PRECISION: u64 = PERCENTAGE_PRECISION as u64; // expo = -6
pub const BID_ASK_SPREAD_PRECISION_I64: i64 = (BID_ASK_SPREAD_PRECISION) as i64;
pub const BID_ASK_SPREAD_PRECISION_U128: u128 = BID_ASK_SPREAD_PRECISION as u128; // expo = -6
pub const BID_ASK_SPREAD_PRECISION_I128: i128 = BID_ASK_SPREAD_PRECISION as i128; // expo = -6

// pub const IF_FACTOR_PRECISION: u128 = PERCENTAGE_PRECISION; // expo 6

// Represents the utilization of the lending borrowing pool that is used for accessing spot margin
pub const SPOT_UTILIZATION_PRECISION: u128 = PERCENTAGE_PRECISION; // expo = -6
pub const SPOT_UTILIZATION_PRECISION_U32: u32 = PERCENTAGE_PRECISION as u32; // expo = -6

// Precision for interest rates on lending and borrowing on spot margin
pub const SPOT_RATE_PRECISION: u128 = PERCENTAGE_PRECISION;
pub const SPOT_RATE_PRECISION_U32: u32 = PERCENTAGE_PRECISION as u32;

// Refers to the precision of fees incurred during liquidation of an asset.
pub const LIQUIDATION_FEE_PRECISION_U32: u32 = PERCENTAGE_PRECISION as u32;
pub const LIQUIDATION_FEE_PRECISION_U128: u128 = LIQUIDATION_FEE_PRECISION_U32 as u128;

// IMF precisions. IMF also referred to as initial margin factor, inspired from FTX risk engine, which essentially discounts
// the initial margin based on size of the position, in order to reduce the impact of concentration of whales. Higher the order
// size, lower the amount of initial margin available. 
pub const IMF_PRECISION: u32 = PERCENTAGE_PRECISION as u32;
pub const IMF_PRECISION_U128: u128 = IMF_PRECISION as u128;

// Precision conversions
pub const PRICE_TO_QUOTE_PRECISION_RATIO: u128 = PRICE_PRECISION/QUOTE_PRECISION; // exponent = -1
pub const LIQUIDATION_FEE_TO_MARGIN_PRECISION_RATIO: u32 = LIQUIDATION_FEE_PRECISION_U32/MARGIN_PRECISION;
pub const LIQUIDATION_FEE_TO_MARGIN_PRECISION_RATION_U128: u128 = LIQUIDATION_FEE_TO_MARGIN_PRECISION_RATIO as u128;

// TODO: ADD PRECISIONS FOR FEE REBATES

// Precision for quote amounts
// For exaaple: ONE_HUNDRED_MILLION_QUOTE refers to one hundred million dollars worth of the quote asset
pub const ONE_HUNDRED_MILLION_QUOTE: u64 = 100_000_000_u64 * QUOTE_PRECISION_U64;
pub const FIFTY_MILLION_QUOTE: u64 = 50_000_000_u64 * QUOTE_PRECISION_U64;
pub const TEN_MILLION_QUOTE: u64 = 10_000_000_u64 * QUOTE_PRECISION_U64;
pub const FIVE_MILLION_QUOTE: u64 = 10_000_000_u64 * QUOTE_PRECISION_U64;
pub const ONE_MILLION_QUOTE: u64 = 1_000_000_u64 * QUOTE_PRECISION_U64;
pub const TWO_HUNDRED_FIFTY_THOUSAND_QUOTE: u64 = 250_000_u64 * QUOTE_PRECISION_U64;
pub const ONE_HUNDRED_THOUSAND_QUOTE: u64 = 100_000_u64 * QUOTE_PRECISION_U64;
pub const TWENTY_FIVE_THOUSAND_QUOTE: u64 = 25_000_u64 * QUOTE_PRECISION_U64;
pub const TEN_THOUSAND_QUOTE: u64 = 10_000_u64 * QUOTE_PRECISION_U64;
pub const ONE_THOUSAND_QUOTE: u64 = 1_000_u64 * QUOTE_PRECISION_U64;
pub const TWO_HUNDRED_FIFTY_QUOTE: u64 = 250_u64 * QUOTE_PRECISION_U64;

// Precision for time periods
pub const ONE_MINUTE: i128 = 60_i128;
pub const FIVE_MINUTE: i128 = (60 * 5) as i128;
pub const ONE_HOUR: i64 = 3600;
pub const ONE_HOUR_I128: i128 = ONE_HOUR as i128;
pub const TWENTY_FOUR_HOUR: i64 = 3600 * 24;
pub const THIRTEEN_DAY: i64 = TWENTY_FOUR_HOUR * 13; // IF unstake default
pub const EPOCH_DURATION: i64 = TWENTY_FOUR_HOUR * 28;
pub const THIRTY_DAY: i64 = TWENTY_FOUR_HOUR * 30;
pub const THIRTY_DAY_I128: i128 = (TWENTY_FOUR_HOUR * 30) as i128;
pub const ONE_YEAR: u128 = 31536000;

// TODO: ADD PRECISIONS FOR INSURANCE FUND

// Quote asset thresholds
pub const FEE_POOL_TO_REVENUE_POOL_THRESHOLD: u128 = TWO_HUNDRED_FIFTY_QUOTE as u128;

// Fee precision
pub const ONE_BPS_DENOMINATOR: u32 = 10000; // if fees are in single digit bps, then denominator => 10000
pub const FEE_DENOMINATOR: u32 = 10*ONE_BPS_DENOMINATOR; 
pub const FEE_PERCENTAGE_DENOMINATOR: u32 = 100; // for eg: fees = 4%, denominator => 100

// Price amounts
pub const HUNDREDTH_OF_CENT: u128 = PRICE_PRECISION/10_000;

// Slippage while liquidation
pub const MAX_LIQUIDATION_SLIPPAGE: i128 = 10_000; // exponent = -2
pub const MAX_LIQUIDATION_SLIPPAGE_U128: u128 = 10_000;

// Refers to the max divergence that we can afford between the mark price and the TWAP of the oracle
pub const MAX_MARK_TWAP_DIVERGENCE: u128 = 500_000; // exponent = -3

// Refers to the minimum and maximum margin that one can put up as collateral
pub const MAX_MARGIN_RATIO: u32 = MARGIN_PRECISION as u32; // 1x or no leverage
pub const MIN_MARGIN_RATIO: u32 = MARGIN_PRECISION as u32 /50; // max leverage, 50x leverage

// Refers to the maximum positive upnl supported for calculation of the initial margin. Higher the upnl, lower the
// initial margin. as we do not want traders using these paper/temporary profits to use more leverage. the maximum aspect
// refers to the maximum upnl that the formula of calculation of initial margin can take.
pub const MAX_POSITIVE_UPNL_FOR_INITIAL_MARGIN: i128 = 100 * QUOTE_PRECISION_I128; 

// defaults

// Refers to the maximum divergence of the TWAP from the current TWAP, which is 1/3
pub const DEFAULT_MAX_TWAP_UPDATE_PRICE_BAND_DENOMINATOR: i64 = 3;

// Refers to the maximum bid-ask spread
pub const DEFAULT_LARGE_BID_ASK_FACTOR: u64 = 10 * BID_ASK_SPREAD_PRECISION;

// For the calculation of the minimum amount of balance required to be held in our account to keep the position open,
// we use a maintenance margin + a liquidation buffer ratio formula. This constant is for the calculation of the 
// liquidation buffer ratio.
pub const DEFAULT_LIQUIDATION_MARGIN_BUFFER_RATIO: u32 = (MARGIN_PRECISION as u32) / 50; // 2%

// The smallest amount of base asset representable
pub const DEFAULT_BASE_ASSET_AMOUNT_STEP_SIZE: u64 = BASE_PRECISION_U64 / 10000; // 1e-4;

// Tick size basically refers to the price increment of the market
pub const DEFAULT_QUOTE_ASSET_AMOUNT_TICK_SIZE: u64 =
    PRICE_PRECISION_U64 / DEFAULT_BASE_ASSET_AMOUNT_STEP_SIZE; // 1e-2

// Withdraws
pub const SPOT_MARKET_TOKEN_TWAP_WINDOW: i64 = TWENTY_FOUR_HOUR;