use std::ops;
use std::fmt;
use std::ops::Mul;
use serde::{Serialize, Deserialize};
use crate::fix_float::{FIX_FRACTION_BITS, FIX_ZERO, FixConvert, FixFloat, FixMath};
use crate::f32_vec::F32Vec;
use crate::f64_vec::F64Vec;
use crate::point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FixVec {
    pub x: FixFloat,
    pub y: FixFloat,
}

impl FixVec {
    pub const ZERO: Self = Self { x: FIX_ZERO, y: FIX_ZERO };

    pub fn is_zero(self) -> bool {
        self.x == FIX_ZERO && self.y == FIX_ZERO
    }

    pub fn to_f32vec(self) -> F32Vec {
        F32Vec::new(self.x.f32(), self.y.f32())
    }

    pub fn to_f64vec(self) -> F64Vec {
        F64Vec::new(self.x.f64(), self.y.f64())
    }

    pub fn like_f32vec(self) -> F32Vec {
        F32Vec::new(self.x as f32, self.y as f32)
    }

    pub fn like_f64vec(self) -> F64Vec {
        F64Vec::new(self.x as f64, self.y as f64)
    }

    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn new_f64(x: f64, y: f64) -> Self {
        Self { x: x.fix(), y: y.fix() }
    }

    pub fn new_f32(x: f32, y: f32) -> Self {
        Self { x: x.fix(), y: y.fix() }
    }

    pub fn new_number(x: i64, y: i64) -> Self {
        Self { x: x.fix(), y: y.fix() }
    }

    pub fn new_point(point: Point) -> Self {
        Self { x: point.x as i64, y: point.y as i64 }
    }

    pub fn fix_sqr_length(self) -> FixFloat {
        self.sqr_length() >> FIX_FRACTION_BITS
    }

    pub fn fix_length(self) -> FixFloat {
        self.length()
    }

    pub fn fix_normalize(self) -> Self {
        let l = self.length();
        let s = (1 << 30) / l;
        let x = (s * self.x).fix_sqr_normalize();
        let y = (s * self.y).fix_sqr_normalize();

        Self { x, y }
    }

    pub fn fix_safe_normalize(self) -> Self {
        self.fix_normalize_with_def_value(Self::new_number(0, 1))
    }

    pub fn fix_normalize_with_def_value(self, def: Self) -> Self {
        if self.is_zero() {
            return def;
        }
        self.fix_normalize()
    }

    pub fn fix_dot_product(self, v: Self) -> FixFloat { // dot product (cos)
        let xx = self.x.fix_mul(v.x);
        let yy = self.y.fix_mul(v.y);
        xx + yy
    }

    pub fn fix_cross_product(self, v: Self) -> FixFloat { // cross product
        let a = self.x.fix_mul(v.y);
        let b = self.y.fix_mul(v.x);

        a - b
    }

    pub fn fix_cross_product_scalar(self, a: FixFloat) -> Self { // cross product
        let x0 = a.fix_mul(self.y);
        let y0 = a.fix_mul(self.x);

        Self::new(-x0, y0)
    }

    pub fn fix_sqr_distance(self, v: Self) -> FixFloat {
        (self - v).fix_sqr_length()
    }

    pub fn fix_distance(self, v: Self) -> FixFloat {
        self.fix_sqr_distance(v).sqrt()
    }

    pub fn half(self) -> FixVec {
        FixVec::new(self.x / 2, self.y / 2)
    }

    pub fn middle(self, v: Self) -> FixVec {
        let sum = self + v;
        Self::new(sum.x / 2, sum.y / 2)
    }

    pub fn sqr_length(self) -> i64 {
        self.x.sqr() + self.y.sqr()
    }

    pub fn length(self) -> FixFloat {
        self.sqr_length().sqrt()
    }

    pub fn dot_product(self, v: Self) -> i64 { // dot product (cos)
        let xx = self.x * v.x;
        let yy = self.y * v.y;
        xx + yy
    }

    pub fn cross_product(self, v: Self) -> i64 { // cross product
        let a = self.x * v.y;
        let b = self.y * v.x;

        a - b
    }

    pub fn sqr_distance(self, v: Self) -> i64 {
        (self - v).sqr_length()
    }
}

impl Mul<i64> for FixVec {
    type Output = FixVec;

    fn mul(self, scalar: i64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl ops::Add for FixVec {
    type Output = FixVec;

    fn add(self, other: FixVec) -> FixVec {
        FixVec {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for FixVec {
    type Output = FixVec;

    fn sub(self, other: FixVec) -> FixVec {
        FixVec {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl fmt::Display for FixVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}