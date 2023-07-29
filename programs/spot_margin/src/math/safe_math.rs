use crate::{
    error::{
        SpedXSpotResult,
        ErrorCode,
    },
    math::{
        bignumber::{
            U192,
            U256
        },
        ceil_div::CheckedCeilDiv,
        floor_div::CheckedFloorDiv
    }
};
use solana_program::msg;
use std::panic::Location;

pub trait SafeMath: Sized {
    /// Function add two numbers, and return None if there is an underflow or an overflow
    fn safe_add(self, rhs: Self) -> SpedXSpotResult<Self>;

    /// Function subtract two numbers, and return None if there is an underflow or an overflow
    fn safe_sub(self, rhs: Self) -> SpedXSpotResult<Self>;

    /// Function multiply two numbers, and return None if there is an underflow or an overflow
    fn safe_mul(self, rhs: Self) -> SpedXSpotResult<Self>;

    /// Function to perform division, and return None if there is an underflow or an overflow or division by 0
    fn safe_div(self, rhs: Self) -> SpedXSpotResult<Self>;

    /// Function to perform ceiling division, and return None if there is an underflow or an overflow or division by 0
    fn safe_ceil_div(self, rhs: Self) -> SpedXSpotResult<Self>;
}

macro_rules! checked_impl {
    ($t:ty) => {
        impl SafeMath for $t {
            /// Implemehting safe_add for type $t. We add self to $t, and if a result is obtained, we return it.
            /// We add both the variables using the checked_add function in rust.
            #[track_caller]
            #[inline(always)]
            fn safe_add(self, v: $t) -> SpedXSpotResult<$t> {
                match self.checked_add(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Addition error thrown at {}:{}", caller.file(), caller.line());
                        Err(
                            ErrorCode::MathError
                        )
                    }
                }
            }

            /// Implementing safe_sub for type $t. We subtract $t from self, and if a result is obtained, we return it.
            /// We subtract the variables using the checked_sub function in rust.
            #[track_caller]
            #[inline(always)]
            fn safe_sub(self, v: $t) -> SpedXSpotResult<$t> {
                match self.checked_sub(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Subtraction error thrown at {}:{}", caller.file(), caller.line());
                        Err(
                            ErrorCode::MathError
                        )
                    }
                }
            }

            /// Implementing safe_div for type $t. We multiply self by $t, and if a result is obtained, we return it.
            /// We multiply the variables using the checked_div function in rust.
            #[track_caller]
            #[inline(always)]
            fn safe_mul(self, v: $t) -> SpedXSpotResult<$t> {
                match self.checked_mul(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Multiplication error thrown at {}:{}", caller.file(), caller.line());
                        Err(
                            ErrorCode::MathError
                        )
                    }
                }
            }

            /// Implementing safe_div for type $t. We divide self by $t, and if a result is obtained, we return it.
            /// We divide the variables using the checked_div function in rust.
            #[track_caller]
            #[inline(always)]
            fn safe_div(self, v: $t) -> SpedXSpotResult<$t> {
                match self.checked_div(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Division error thrown at {}:{}", caller.file(), caller.line());
                        Err(
                            ErrorCode::MathError
                        )
                    }
                }
            }

            /// Implementing safe_ceil_div for type $t. We divide self by $t, and if a result is obtained, we return it.
            /// We divide the variables using the checked_ceil_div function in rust.
            #[track_caller]
            #[inline(always)]
            fn safe_ceil_div(self, v: $t) -> SpedXSpotResult<$t> {
                match self.checked_ceil_div(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Ceiling division error thrown at {}:{}", caller.file(), caller.line());
                        Err(
                            ErrorCode::MathError
                        )
                    }
                }
            }
        }
    };
}

checked_impl!(U256);
checked_impl!(U192);
checked_impl!(u128);
checked_impl!(u64);
checked_impl!(u32);
checked_impl!(u16);
checked_impl!(u8);
checked_impl!(i128);
checked_impl!(i64);
checked_impl!(i32);
checked_impl!(i16);
checked_impl!(i8);

pub trait SafeFloorDiv: Sized{ 
    /// Performs floor division
    fn safe_floor_div(self, rhs: Self) -> SpedXSpotResult<Self>;    
}

macro_rules! div_floor_impl {
    ($t:ty) => {
        impl SafeFloorDiv for $t {

            /// Implementing safe_floor_div for type $t. We divide self by $t, and if a result is obtained, we return it.
            /// We divide the variables using the checked_floor_div function in rust.
            #[track_caller]
            #[inline(always)]
            fn safe_floor_div(self, v: $t) -> SpedXSpotResult<$t> {
                match self.checked_floor_div(v) {
                    Some(result) => Ok(result),
                    None => {
                        let caller = Location::caller();
                        msg!("Floor division error thrown at {}:{}", caller.file(), caller.line());
                        Err(
                            ErrorCode::MathError
                        )
                    }
                }
            }
        }
    };
}

div_floor_impl!(i128);
div_floor_impl!(i64);
div_floor_impl!(i32);
div_floor_impl!(i16);
div_floor_impl!(i8);

#[cfg(test)]
mod test {
    use crate::{
        error::ErrorCode,
        math::safe_math::{
            SafeFloorDiv,
            SafeMath,
        }
    };

    /// Test for safe_add
    #[test]
    fn safe_add() {
        assert_eq!(1_u128.safe_add(1).unwrap(), 2);
        assert_eq!(1_u128.safe_add(u128::MAX), Err(ErrorCode::MathError));
    }

    #[test]
    /// Test for safe_sub
    fn safe_sub() {
        assert_eq!(1_u128.safe_sub(1).unwrap(), 0);
        assert_eq!(0_u128.safe_sub(1), Err(ErrorCode::MathError));
    }

    #[test]
    /// Test for safe_mul
    fn safe_mul() {
        assert_eq!(8_u128.safe_mul(8).unwrap(), 64);
        assert_eq!(2_u128.safe_mul(u128::MAX), Err(ErrorCode::MathError));
    }

    /// Test for safe_div
    #[test]
    fn safe_div() {
        assert_eq!(155_u128.safe_div(8).unwrap(), 19);
        assert_eq!(159_u128.safe_div(8).unwrap(), 19);
        assert_eq!(160_u128.safe_div(8).unwrap(), 20);
        assert_eq!(1_u128.safe_div(100).unwrap(), 0);
        assert_eq!(1_u128.safe_div(0), Err(ErrorCode::MathError));
    }

    /// Test for safe_floor_div
    #[test]
    fn safe_floor_div() {
        assert_eq!((-155_i128).safe_floor_div(8).unwrap(), -20);
        assert_eq!((-159_i128).safe_floor_div(8).unwrap(), -20);
        assert_eq!((-160_i128).safe_floor_div(8).unwrap(), -20);
    }

}
