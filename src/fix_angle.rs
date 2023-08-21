use std::ops;
use std::f64;
use std::f32;
use crate::fix_float::FixFloat;
use crate::fix_sin::FixSin;
use crate::fix_vec::FixVec;

// split 90 grad to 128 parts, 360 will be 512
#[derive(Debug)]
pub struct FixAngle(i64);

impl FixAngle {
    pub const INDEX_MASK: i64 = 256 - 1;
    pub const FULL_ROUND_MASK: i64 = 1024 - 1;
    pub const F64_TO_ANGLE: f64 = 1024.0 * 512.0 / f64::consts::PI;
    pub const F32_TO_ANGLE: f32 = 1024.0 * 512.0 / f32::consts::PI;

    pub fn trim(&self) -> i64 {
        self.0 & Self::FULL_ROUND_MASK
    }

    pub fn radian(&self) -> f64 {
        self.trim() as f64 * Self::F64_TO_ANGLE
    }

    pub fn sin(&self) -> FixFloat {
        let quarter = ((self.0 & Self::FULL_ROUND_MASK) >> 8) as usize;
        let index = (self.0 & Self::INDEX_MASK) as usize;

        match quarter {
            0 => FixFloat::new_i64(Self::sin_by_index(index)),
            1 => FixFloat::new_i64(Self::sin_by_index(256 - index)),
            2 => FixFloat::new_i64(-Self::sin_by_index(index)),
            _ => FixFloat::new_i64(-Self::sin_by_index(256 - index))
        }
    }

    pub fn cos(&self) -> FixFloat {
        let quarter = ((self.0 & Self::FULL_ROUND_MASK) >> 8) as usize;
        let index = (self.0 & Self::INDEX_MASK) as usize;

        match quarter {
            0 => FixFloat::new_i64(Self::sin_by_index(256 - index)),
            1 => FixFloat::new_i64(-Self::sin_by_index(index)),
            2 => FixFloat::new_i64(-Self::sin_by_index(256 - index)),
            _ => FixFloat::new_i64(Self::sin_by_index(index))
        }
    }

    pub fn rotator(&self) -> FixVec {
        let quarter = ((self.0 & Self::FULL_ROUND_MASK) >> 8) as usize;
        let index = (self.0 & Self::INDEX_MASK) as usize;

        match quarter {
            0 => FixVec::new_i64(Self::sin_by_index(index), Self::sin_by_index(256 - index)),
            1 => FixVec::new_i64(Self::sin_by_index(256 - index), -Self::sin_by_index(index)),
            2 => FixVec::new_i64(-Self::sin_by_index(index), -Self::sin_by_index(256 - index)),
            _ => FixVec::new_i64(-Self::sin_by_index(256 - index), Self::sin_by_index(index))
        }
    }

    fn sin_by_index(index: usize) -> i64 {
        let i = index >> 1;
        if index & 1 == 1 {
            (FixSin::value(i) + FixSin::value(i + 1)) >> 1
        } else {
            FixSin::value(i)
        }
    }
}
impl ops::Add for FixAngle {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        FixAngle(self.0 + other.0)
    }
}

impl ops::Sub for FixAngle {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        FixAngle(self.0 - other.0)
    }
}