use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UInt128 {
    pub high: u64,
    pub low: u64,
}

impl UInt128 {
    #[inline(always)]
    pub fn new(high: u64, low: u64) -> Self {
        Self { high, low }
    }

    #[inline]
    fn sum(a: u64, b: u64, c: u64) -> (u64, u64) {
        let (s0, overflow0) = a.overflowing_add(b);
        let mut high = if overflow0 { 1 } else { 0 };
        let (s1, overflow1) = s0.overflowing_add(c);
        if overflow1 { high += 1; }

        (s1, high)
    }

    #[inline]
    pub fn multiply(a: u64, b: u64) -> Self {
        if a.leading_zeros() + b.leading_zeros() >= u64::BITS {
            return Self::new(0, a * b);
        }

        let a1 = a >> 32;
        let a0 = a & 0xFFFF_FFFF;
        let b1 = b >> 32;
        let b0 = b & 0xFFFF_FFFF;

        let ab00 = a0 * b0;
        let (m_partial, m_high) = Self::sum(a0 * b1, a1 * b0, ab00 >> 32);
        let high = a1 * b1 + (m_partial >> 32) + (m_high << 32);

        let low = (m_partial << 32) | (ab00 & 0xFFFF_FFFF);

        Self::new(high, low)
    }
}

impl PartialOrd for UInt128 {

    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UInt128 {

    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_high = self.high.cmp(&other.high);
        match cmp_high {
            Ordering::Equal => {
                self.low.cmp(&other.low)
            }
            _ => {
                cmp_high
            }
        }
    }
}