use core::f64;
use core::f32;
use crate::fix_float::{FIX_FRACTION_BITS, FIX_PI, FixFloat};
use crate::fix_sin::FixSin;
use crate::fix_vec::FixVec;

// split 90 grad to 128 parts, 360 will be 512
pub type FixAngle = i64;

pub const FIX_ANGLE_INDEX_MASK: i64 = 256 - 1;
pub const FIX_ANGLE_FULL_ROUND_MASK: i64 = 1024 - 1;
pub const FIX_ANGLE_F64_TO_ANGLE: f64 = 1024.0 * 512.0 / f64::consts::PI;
pub const FIX_ANGLE_F32_TO_ANGLE: f32 = 1024.0 * 512.0 / f32::consts::PI;
pub const FIX_ANGLE_F64_TO_RADIAN: f64 = f64::consts::PI / 512.0;
pub const FIX_ANGLE_F32_TO_RADIAN: f32 = f32::consts::PI / 512.0;

pub trait FixTrigonometry {
    fn new_from_radians_f64(radians: f64) -> Self;
    fn new_from_radians_f32(radians: f32) -> Self;
    fn new_from_radians_fix(radians: FixFloat) -> Self;
    fn new_from_degrees_f64(degrees: f64) -> Self;
    fn new_from_degrees_f32(degrees: f32) -> Self;
    fn new_from_degrees_fix(degrees: FixFloat) -> Self;
    fn trim(&self) -> Self;
    fn radians_f64(&self) -> f64;
    fn radians_f32(&self) -> f32;
    fn sin(&self) -> FixFloat;
    fn cos(&self) -> FixFloat;
    fn rotator(&self) -> FixVec;
}

impl FixTrigonometry for FixAngle {
    #[inline(always)]
    fn new_from_radians_f64(radians: f64) -> Self {
        (radians * FIX_ANGLE_F64_TO_ANGLE) as i64 >> FIX_FRACTION_BITS
    }

    #[inline(always)]
    fn new_from_radians_f32(radians: f32) -> Self {
        (radians * FIX_ANGLE_F32_TO_ANGLE) as i64 >> FIX_FRACTION_BITS
    }

    #[inline(always)]
    fn new_from_radians_fix(radians: FixFloat) -> Self {
        (radians << 9) / FIX_PI
    }

    #[inline(always)]
    fn new_from_degrees_f64(degrees: f64) -> Self {
        (degrees * 1024.0 / 360.0) as i64
    }

    #[inline(always)]
    fn new_from_degrees_f32(degrees: f32) -> Self {
        (degrees * 1024.0 / 360.0) as i64
    }

    #[inline(always)]
    fn new_from_degrees_fix(degrees: FixFloat) -> Self {
        degrees / 360
    }

    #[inline(always)]
    fn trim(&self) -> i64 {
        self & FIX_ANGLE_FULL_ROUND_MASK
    }

    #[inline(always)]
    fn radians_f64(&self) -> f64 {
        self.trim() as f64 * FIX_ANGLE_F64_TO_RADIAN
    }

    #[inline(always)]
    fn radians_f32(&self) -> f32 {
        self.trim() as f32 * FIX_ANGLE_F32_TO_RADIAN
    }

    #[inline(always)]
    fn sin(&self) -> FixFloat {
        let quarter = ((self & FIX_ANGLE_FULL_ROUND_MASK) >> 8) as usize;
        let index = (self & FIX_ANGLE_INDEX_MASK) as usize;

        match quarter {
            0 => sin_by_index(index),
            1 => sin_by_index(256 - index),
            2 => -sin_by_index(index),
            _ => -sin_by_index(256 - index)
        }
    }

    #[inline(always)]
    fn cos(&self) -> FixFloat {
        let quarter = ((self & FIX_ANGLE_FULL_ROUND_MASK) >> 8) as usize;
        let index = (self & FIX_ANGLE_INDEX_MASK) as usize;

        match quarter {
            0 => sin_by_index(256 - index),
            1 => -sin_by_index(index),
            2 => -sin_by_index(256 - index),
            _ => sin_by_index(index)
        }
    }

    #[inline(always)]
    fn rotator(&self) -> FixVec {
        let quarter = ((self & FIX_ANGLE_FULL_ROUND_MASK) >> 8) as usize;
        let index = (self & FIX_ANGLE_INDEX_MASK) as usize;

        match quarter {
            0 => FixVec::new(sin_by_index(index), sin_by_index(256 - index)),
            1 => FixVec::new(sin_by_index(256 - index), -sin_by_index(index)),
            2 => FixVec::new(-sin_by_index(index), -sin_by_index(256 - index)),
            _ => FixVec::new(-sin_by_index(256 - index), sin_by_index(index))
        }
    }
}

#[inline(always)]
fn sin_by_index(index: usize) -> i64 {
    let i = index >> 1;
    if index & 1 == 1 {
        (FixSin::value(i) + FixSin::value(i + 1)) >> 1
    } else {
        FixSin::value(i)
    }
}