use num_traits::{
    One,
    Zero
};

pub trait CheckedFloorDiv: Sized { 
    /// Performs floor division
    fn checked_floor_div(&self, rhs: Self) -> Option<Self>;
}

macro_rules! checked_impl {
    ($t:ty) => {
        impl CheckedFloorDiv for $t { 
            #[track_caller]
            #[inline]
            fn checked_floor_div(&self, rhs: $t) -> Option<$t> {
                // we divide &self with rhs, using the checked_div function in rust. If there is any underflow, overflow
                // or division by 0, a None value is returned instead of the function panicking. 
                // What is underflow? Underflow refers to a situation wherein a value obtained after an operations is
                // lesser than the minimum acceptable value for the datatype. Overflow is the opposite of underflow.
                let quotient = self.checked_div(rhs)?;

                // Finds the remainder of diving self with rhs
                let remainder = self.checked_rem(rhs)?;

                // if the remainder is not zero, we round down the quotient
                if remainder != <$t>::zero() {
                    quotient.checked_sub(<$t>::one())
                } else {
                    // if the remainder is zero, return the quotient.
                    Some(quotient)
                }
            }
        }
    };
}

checked_impl!(i128);
checked_impl!(i64);
checked_impl!(i32);
checked_impl!(i16);
checked_impl!(i8);

#[cfg(test)]
mod test {
    use crate::math::floor_div::CheckedFloorDiv;

    #[test]
    fn test() {
        // i128 value
        let x = -3_i128;

        // remainder = -1.5, rounded down = -2
        assert_eq!(x.checked_floor_div(2), Some(-2));
        // division by 0 returns None
        assert_eq!(x.checked_floor_div(0), None);
    } 
}