pub type FixFloat = i64;

pub const FIX_FRACTION_BITS: usize = 10;
pub const FIX_SQR_FRACTION_BITS: i64 = 20;
pub const FIX_CUBE_FRACTION_BITS: i64 = 30;
pub const FIX_TETRA_FRACTION_BITS: i64 = 40;
pub const FIX_PENTA_FRACTION_BITS: i64 = 50;

pub const FIX_MAX: i64 = i32::MAX as i64;
pub const FIX_MIN: i64 = i32::MIN as i64;

pub const FIX_ZERO: FixFloat = 0;
pub const FIX_UNIT: i64 = 1 << FIX_FRACTION_BITS;
pub const FIX_SQR_UNIT: i64 = 1 << FIX_SQR_FRACTION_BITS;
pub const FIX_CUBE_UNIT: i64 = 1 << FIX_CUBE_FRACTION_BITS;
pub const FIX_HALF: i64 = 1 << (FIX_FRACTION_BITS - 1);
pub const FIX_PI: i64 = 3217;

pub trait FixMath {
    fn f32(self) -> f32;
    fn f64(self) -> f64;
    fn fix_div(self, value: FixFloat) -> FixFloat;
    fn fix_mul(self, value: FixFloat) -> FixFloat;
    fn fix_sqr(self) -> FixFloat;
    fn fix_sqrt(self) -> FixFloat;

    fn sqr(self) -> FixFloat;
    fn sqrt(self) -> FixFloat;
    fn fix_normalize(self) -> FixFloat;
    fn fix_sqr_normalize(self) -> FixFloat;
}

impl FixMath for FixFloat {
    fn f32(self) -> f32 {
        (self as f32) / (FIX_UNIT as f32)
    }
    fn f64(self) -> f64 {
        (self as f64) / (FIX_UNIT as f64)
    }

    fn fix_div(self, value: FixFloat) -> FixFloat {
        (self << FIX_FRACTION_BITS) / value
    }

    fn fix_mul(self, value: FixFloat) -> FixFloat {
        (self * value) / FIX_UNIT
    }

    fn fix_sqr(self) -> FixFloat {
        (self * self) >> FIX_FRACTION_BITS
    }

    fn fix_sqrt(self) -> FixFloat {
        (self << FIX_FRACTION_BITS).sqrt()
    }

    fn sqr(self) -> FixFloat {
        self * self
    }

    fn sqrt(self) -> FixFloat {
        let a = (self as f64).sqrt() as i64;
        let b = a + 1;

        if b * b > self { a } else { b }
    }

    fn fix_normalize(self) -> FixFloat {
        self / FIX_UNIT
    }

    fn fix_sqr_normalize(self) -> FixFloat {
        self / FIX_SQR_UNIT
    }
}

pub trait FixConvert {
    fn fix(self) -> FixFloat;
}

impl FixConvert for f64 {
    fn fix(self) -> FixFloat {
        (self * (FIX_UNIT as f64)) as FixFloat
    }
}

impl FixConvert for f32 {
    fn fix(self) -> FixFloat {
        (self * (FIX_UNIT as f32)) as FixFloat
    }
}

impl FixConvert for i64 {
    fn fix(self) -> FixFloat {
        self << FIX_FRACTION_BITS
    }
}