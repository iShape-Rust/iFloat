use std::ops;
use std::fmt;
use std::f64;
use std::f32;
use crate::fix_float::FixFloat;
use crate::fix_sin::FixSin;
use crate::fix_vec::FixVec;
use serde::{Serialize, Deserialize};

// split 90 grad to 128 parts, 360 will be 512
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FixAngle(i64);

impl FixAngle {
    pub const INDEX_MASK: i64 = 256 - 1;
    pub const FULL_ROUND_MASK: i64 = 1024 - 1;
    pub const F64_TO_ANGLE: f64 = 1024.0 * 512.0 / f64::consts::PI;
    pub const F32_TO_ANGLE: f32 = 1024.0 * 512.0 / f32::consts::PI;
    pub const F64_TO_RADIAN: f64 = f64::consts::PI / 512.0;
    pub const F32_TO_RADIAN: f32 = f32::consts::PI / 512.0;


    pub fn value(&self) -> i64 {
        self.0
    }

    pub fn new_from_radians_f64(radians: f64) -> Self {
        let value = (radians * Self::F64_TO_ANGLE) as i64 >> FixFloat::FRACTION_BITS;
        FixAngle(value)
    }

    pub fn new_from_radians_f32(radians: f32) -> Self {
        let value = (radians * Self::F32_TO_ANGLE) as i64 >> FixFloat::FRACTION_BITS;
        FixAngle(value)
    }

    pub fn new_from_radians_fix(radians: FixFloat) -> Self {
        let value = (radians.value() << 9) / FixFloat::PI;
        FixAngle(value)
    }

    pub fn new_from_degrees_f64(degrees: f64) -> Self {
        FixAngle((degrees * 1024.0 / 360.0) as i64)
    }

    pub fn new_from_degrees_f32(degrees: f32) -> Self {
        FixAngle((degrees * 1024.0 / 360.0) as i64)
    }

    pub fn new_from_degrees_fix(degrees: FixFloat) -> Self {
        FixAngle(degrees.value() / 360)
    }

    pub fn trim(&self) -> i64 {
        self.0 & Self::FULL_ROUND_MASK
    }

    pub fn radians_f64(&self) -> f64 {
        self.trim() as f64 * Self::F64_TO_RADIAN
    }

    pub fn radians_f32(&self) -> f32 {
        self.trim() as f32 * Self::F32_TO_RADIAN
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

impl fmt::Display for FixAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}