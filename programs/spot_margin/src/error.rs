use anchor_lang::prelude::*;
// use solana_program::msg;

pub type SpedXSpotResult<T = ()> = std::result::Result<T, ErrorCode>;

#[error_code]
#[derive(PartialEq, Eq)]
pub enum ErrorCode {
    #[msg("Unable to load AccountLoader")]
    UnableToLoadAccountLoader,
    #[msg("Unable to convert u128 to u64 due to an underflow or an overflow")]
    BigNumberConversionError,
    #[msg("Unable to cast datatype")]
    CastingFailure,
    #[msg("Math Error: Unable to perform operation")]
    MathError,
    #[msg("Oracle is Invalid")]
    InvalidOracle,
    #[msg("Unable to Load Oracle")]
    UnableToLoadOracle,
    #[msg("Cannot update pool balance type")]
    CannotUpdatePoolBalanceType,
    #[msg("Insurance Fund has not been introduced yet")]
    InsuranceFundNotIntroduced,
    #[msg("Unable to Unwrap")]
    UnwrapError,
    #[msg("Invalid Oracle: Price is Negative/Non-Positive")]
    OracleNegativeError,
    #[msg("Oracle price is too volatile")]
    OracleTooVolatile,
    #[msg("Oracle confidence is too wide than supported limits")]
    OracleConfidenceTooWide,
    #[msg("Oracle price is stale for margin")]
    OraclePriceStaleForMargin,
    #[msg("Unable to cast unix timestamps")]
    UnableToCastUnixTimestamp,
    #[msg("Unable to increment value safely")]
    SafeIncrementError,
    #[msg("Unable to decrement value safely")]
    SafeDecrementError,
    #[msg("Unable to use specified oracle")]
    OracleNotFound,
    #[msg("The spread between the oracle and the limit price for oracle offset limit orders has driven down the price to 0")]
    InvalidOracleSpreadLimitPrice,
    #[msg("Unable to fetch limit price")]
    UnableToGetLimitPrice
}

// #[macro_export]
// macro_rules! math_error {
//     () => {{
//         || {
//             let error_code = $crate::error::ErrorCode::MathError;
//             msg!("Error {} thrown at {}:{}", error_code, file!(), line!());
//             error_code
//         }
//     }};
// }