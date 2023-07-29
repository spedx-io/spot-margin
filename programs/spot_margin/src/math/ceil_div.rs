use crate::math::bignumber::{
    U192,
    U256
};
use num_traits::{
    One,
    Zero,
};

pub trait CheckedCeilDiv: Sized {
    /// Perform ceiling division
    fn checked_ceil_div(&self, rhs: Self) -> Option<Self>;
}

macro_rules! checked_impl {
    ($t:ty) => {
        impl CheckedCeilDiv for $t {
            // we perform
            #[track_caller]
            #[inline]
            fn checked_ceil_div(&self, rhs: $t) -> Option<$t> {
                // we divide &self with rhs, using the checked_div function in rust. If underflow, overflow or division by 
                // 0 happens, a None value is returned
                let quotient = self.checked_div(rhs)?;

                // finds the remainder of dividing self with the rhs
                let remainder = self.checked_rem(rhs)?;

                // if the remainder is more than zero, round a number up.
                if remainder > <$t>::zero() {
                    quotient.checked_add(<$t>::one())
                } else {
                    Some(quotient)
                }
            }
        }
    };
}

// unsigned integers
checked_impl!(U256);
checked_impl!(U192);
checked_impl!(u128);
checked_impl!(u64);
checked_impl!(u32);
checked_impl!(u16);
checked_impl!(u8);

// signed integers
checked_impl!(i128);
checked_impl!(i64);
checked_impl!(i32);
checked_impl!(i16);
checked_impl!(i8);