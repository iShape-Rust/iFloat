use crate::int::number::product_uint::UIntProduct;
use crate::int::number::uint::UIntNumber;
use crate::int::number::wide_int::WideIntNumber;
use core::cmp::Ordering;
use core::marker::PhantomData;

/// Signed-magnitude value backed by the double-width product of an integer.
#[derive(Clone, Copy)]
pub struct SignedProduct<I: WideIntNumber> {
    magnitude: <I::UInt as UIntNumber>::Product,
    negative: bool,
    int: PhantomData<I>,
}

impl<I: WideIntNumber> SignedProduct<I> {
    /// Multiplies two signed integers without narrowing the result.
    #[inline]
    pub fn multiply(a: I, b: I) -> Self {
        let magnitude = <I::UInt as UIntNumber>::Product::multiply(a.unsigned_abs(), b.unsigned_abs());
        let negative = (a < I::ZERO) != (b < I::ZERO);
        Self::new(magnitude, negative)
    }

    #[inline]
    fn new(magnitude: <I::UInt as UIntNumber>::Product, negative: bool) -> Self {
        let zero = <I::UInt as UIntNumber>::Product::from_uint(I::UInt::ZERO);
        Self {
            magnitude,
            negative: negative && magnitude != zero,
            int: PhantomData,
        }
    }

    /// Adds two signed double-width values, returning `None` on magnitude overflow.
    #[inline]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        if self.negative == other.negative {
            Some(Self::new(
                self.magnitude.checked_add(other.magnitude)?,
                self.negative,
            ))
        } else if self.magnitude >= other.magnitude {
            Some(Self::new(
                self.magnitude.checked_sub(other.magnitude)?,
                self.negative,
            ))
        } else {
            Some(Self::new(
                other.magnitude.checked_sub(self.magnitude)?,
                other.negative,
            ))
        }
    }

    /// Subtracts two signed double-width values, returning `None` on magnitude overflow.
    #[inline]
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_add(Self::new(other.magnitude, !other.negative))
    }

    #[inline]
    pub fn magnitude(&self) -> <I::UInt as UIntNumber>::Product {
        self.magnitude
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.sign() == Ordering::Equal
    }

    #[inline]
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    #[inline]
    pub fn sign(&self) -> Ordering {
        if self.magnitude == <I::UInt as UIntNumber>::Product::from_uint(I::UInt::ZERO) {
            Ordering::Equal
        } else if self.negative {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SignedProduct;
    use core::cmp::Ordering;

    #[test]
    fn multiply_preserves_sign_and_full_width() {
        let value = SignedProduct::<i64>::multiply(-4_000_000_000, 4_000_000_000);

        assert_eq!(value.sign(), Ordering::Less);
        assert!(!value.is_zero());
    }

    #[test]
    fn checked_add_combines_and_cancels_signed_products() {
        let positive = SignedProduct::<i32>::multiply(12, 7);
        let negative = SignedProduct::<i32>::multiply(-10, 7);
        let sum = positive.checked_add(negative).unwrap();
        let zero = positive.checked_add(SignedProduct::multiply(-12, 7)).unwrap();

        assert_eq!(sum.sign(), Ordering::Greater);
        assert_eq!(zero.sign(), Ordering::Equal);
        assert!(!zero.is_negative());
    }

    #[test]
    fn checked_subtracts_signed_products() {
        let a = SignedProduct::<i32>::multiply(12, 7);
        let b = SignedProduct::<i32>::multiply(10, 7);
        let difference = a.checked_sub(b).unwrap();
        let negative = b.checked_sub(a).unwrap();

        assert_eq!(difference.sign(), Ordering::Greater);
        assert_eq!(negative.sign(), Ordering::Less);
    }

    #[test]
    fn checked_add_reports_magnitude_overflow() {
        let value = SignedProduct::<i32>::multiply(i32::MIN, i32::MIN);
        let doubled = value.checked_add(value).unwrap();

        assert!(doubled.checked_add(doubled).is_none());
    }
}
