use crate::int::number::uint::UIntNumber;
use core::cmp::Ordering;

pub trait ExtUIntNumber<U: UIntNumber>: Copy {
    fn multiply(a: U, b: U) -> Self;
}

#[derive(Debug, Copy, Clone)]
pub struct ExtUIntNumber64 {
    pub value: u64,
}

impl<U: UIntNumber> ExtUIntNumber<U> for ExtUIntNumber64 {
    #[inline(always)]
    fn multiply(a: U, b: U) -> Self {
        let value = a.to_u64() * b.to_u64();
        Self { value }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CompositeExtUIntNumber<U: UIntNumber> {
    high: U,
    low: U,
}

impl<U: UIntNumber> CompositeExtUIntNumber<U> {
    #[inline(always)]
    fn new(high: U, low: U) -> Self {
        Self { high, low }
    }

    #[inline]
    fn sum(a: U, b: U, c: U) -> (U, U) {
        let (s0, overflow0) = a.overflowing_add(b);
        let mut high = if overflow0 { U::ONE } else { U::ZERO };
        let (s1, overflow1) = s0.overflowing_add(c);
        if overflow1 {
            high += U::ONE;
        }

        (s1, high)
    }
}
impl<U: UIntNumber> ExtUIntNumber<U> for CompositeExtUIntNumber<U> {
    #[inline]
    fn multiply(a: U, b: U) -> Self {
        if a.leading_zeros() + b.leading_zeros() >= U::BITS {
            return Self::new(U::ZERO, a * b);
        }

        let a1 = a >> U::HALF_BITS;
        let a0 = a & U::HALF_MASK;
        let b1 = b >> U::HALF_BITS;
        let b0 = b & U::HALF_MASK;

        let ab00 = a0 * b0;
        let (m_partial, m_high) = Self::sum(a0 * b1, a1 * b0, ab00 >> U::HALF_BITS);
        let high = a1 * b1 + (m_partial >> U::HALF_BITS) + (m_high << U::HALF_BITS);

        let low = (m_partial << U::HALF_BITS) | (ab00 & U::HALF_MASK);

        Self::new(high, low)
    }
}

impl<U: UIntNumber> PartialOrd for CompositeExtUIntNumber<U> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<U: UIntNumber> Ord for CompositeExtUIntNumber<U> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_high = self.high.cmp(&other.high);
        match cmp_high {
            Ordering::Equal => self.low.cmp(&other.low),
            _ => cmp_high,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::int::number::ext_uint::{CompositeExtUIntNumber, ExtUIntNumber};

    #[test]
    fn test_basic() {
        let result = CompositeExtUIntNumber::<u64>::multiply(2, 3);
        assert_eq!(result.high, 0);
        assert_eq!(result.low, 6);
    }

    #[test]
    fn test_overflow() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0x1_0000_0000, 0x1_0000_0000);
        assert_eq!(result.high, 1);
        assert_eq!(result.low, 0);
    }

    #[test]
    fn test_max() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(result.high, 0xFFFF_FFFF_FFFF_FFFE);
        assert_eq!(result.low, 1);
    }

    #[test]
    fn test_zero() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0, 1234567890);
        assert_eq!(result.high, 0);
        assert_eq!(result.low, 0);
    }

    #[test]
    fn test_one() {
        let result = CompositeExtUIntNumber::<u64>::multiply(1, 1234567890);
        assert_eq!(result.high, 0);
        assert_eq!(result.low, 1234567890);
    }

    #[test]
    fn test_0() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0xFFFF_0000_FFFF_FFFF, 0xFFFF_FFFF_0000_FFFF);
        assert_eq!(result.high, 0xFFFF_0000_0001_FFFC);
        assert_eq!(result.low, 0x1_FFFF_FFFF_0001);
    }

    #[test]
    fn test_1() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0x825e0a1f447a9d0f, 0xbeae05eb50b368cd);
        assert_eq!(result.high, 0x611a6a71c1b2333b);
        assert_eq!(result.low, 0x967f7971277add03);
    }

    #[test]
    fn test_2() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0xa40f0cc4738525b, 0xc4339113aff1fb8);
        assert_eq!(result.high, 0x7dbc91bf17af89);
        assert_eq!(result.low, 0x76583e40a9193668);
    }

    #[test]
    fn test_3() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0x013d10e9cab6d9101, 0x0ac718b6798f0cc2b);
        assert_eq!(result.high, 0xd593fe33e37ff5f);
        assert_eq!(result.low, 0xc8adf423a3e4272b);
    }

    #[test]
    fn test_4() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0xfb1d552bec078d70, 0xcf842b7995bb80d0);
        assert_eq!(result.high, 0xcb8e5da39f8b7104);
        assert_eq!(result.low, 0xcbf4a27a0daaeb00);
    }

    #[test]
    fn test_5() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0x38f44d557e6d9bc0, 0xb5f343ebf6828e7f);
        assert_eq!(result.high, 0x287ad9af49e2acce);
        assert_eq!(result.low, 0x86a08a1e1c44c440);
    }

    #[test]
    fn test_6() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0x5522e2b50ba73069, 0x13bdc5312abbf74);
        assert_eq!(result.high, 0x690b32901dff58);
        assert_eq!(result.low, 0xcab645dabd034694);
    }

    #[test]
    fn test_7() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0xfa2d1b0f09047b2a, 0xb0d6f746db94b662);
        assert_eq!(result.high, 0xacd115f5b8cb70b0);
        assert_eq!(result.low, 0xf7144b9ac58f0214);
    }

    #[test]
    fn test_8() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0x24346c19605512a6, 0x6ccd292ea2c6cb3);
        assert_eq!(result.high, 0xf63216842b6cea);
        assert_eq!(result.low, 0xaec56ab92fe21212);
    }

    #[test]
    fn test_9() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0xe9ffe90c7f3adc66, 0xebc8a042d873cba1);
        assert_eq!(result.high, 0xd7854d59960a17c4);
        assert_eq!(result.low, 0xbc51dd72c29b7e26);
    }

    #[test]
    fn test_10() {
        let result = CompositeExtUIntNumber::<u64>::multiply(0x22e0c81bc93e3a9, 0xa036a6c01e0db3);
        assert_eq!(result.high, 0x15d3ef338213a);
        assert_eq!(result.low, 0x65039ef3cbc5c42b);
    }
}
