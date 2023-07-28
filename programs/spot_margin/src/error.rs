use anchor_lang::prelude::*;

pub type SpedXSpotResult<T = ()> = std::result::Result<T, ErrorCode>;

#[error_code]
#[derive(PartialEq, Eq)]
pub enum ErrorCode {
    #[msg("Unable to load AccountLoader")]
    UnableToLoadAccountLoader,
    #[msg("Unable to convert u128 to u64 due to an underflow or an overflow")]
    BigNumberConversionError,
}