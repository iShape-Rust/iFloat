use crate::fix_vec::FixVec;

pub type BitPack = u64;

const FIX_MID: i64 = 1 << 31;
const Y_MASK: u64 = u32::MAX as u64;

pub trait BitPackVec {
    fn bit_pack(&self) -> BitPack;
}

pub trait BitPackFix {
    fn fix_vec(&self) -> FixVec;
}

impl BitPackVec for FixVec {
    fn bit_pack(&self) -> BitPack {
        let xx = ((self.x + FIX_MID) as u64) << 32;
        let yy = (self.y + FIX_MID) as u64;

        return xx | yy;
    }
}

impl BitPackFix for BitPack {
    fn fix_vec(&self) -> FixVec {
        let x = (self >> 32) as i64 - FIX_MID;
        let y = (self & Y_MASK) as i64 - FIX_MID;

        FixVec::new(x, y)
    }
}