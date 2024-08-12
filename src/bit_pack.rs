use crate::fix_vec::FixVec;
use crate::point::IntPoint;

pub type BitPack = u64;

const FIX_MID: i64 = 1 << 31;
const Y_MASK: u64 = u32::MAX as u64;

pub trait BitPackVec {
    fn bit_pack(&self) -> BitPack;
}

pub trait BitPackFix {
    fn fix_vec(&self) -> FixVec;
    fn x(&self) -> i64;
    fn y(&self) -> i64;
}

impl BitPackVec for FixVec {

    #[inline(always)]
    fn bit_pack(&self) -> BitPack {
        let xx = ((self.x + FIX_MID) as u64) << 32;
        let yy = (self.y + FIX_MID) as u64;

        xx | yy
    }
}

impl BitPackVec for IntPoint {

    #[inline(always)]
    fn bit_pack(&self) -> BitPack {
        let xx = (((self.x as i64) + FIX_MID) as u64) << 32;
        let yy = ((self.y as i64) + FIX_MID) as u64;

        xx | yy
    }
}

impl BitPackFix for BitPack {
    fn fix_vec(&self) -> FixVec {
        FixVec::new(self.x(), self.y())
    }

    fn x(&self) -> i64 {
        (self >> 32) as i64 - FIX_MID
    }

    fn y(&self) -> i64 {
        (self & Y_MASK) as i64 - FIX_MID
    }
}