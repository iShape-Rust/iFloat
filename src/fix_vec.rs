use std::ops;
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::fix_float::{FIX_FRACTION_BITS, FIX_MAX_BITS, FIX_ZERO, FixConvert, FixFloat, FixMath};
use crate::f32_vec::F32Vec;

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

    pub fn bit_pack(self) -> i64 {
        (self.x << FIX_MAX_BITS) + self.y
    }

    pub fn f32vec(self) -> F32Vec {
        F32Vec::new(self.x.f32(), self.y.f32())
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

    pub fn unsafe_sqr_length(self) -> i64 {
        self.x.sqr() + self.y.sqr()
    }

    pub fn sqr_length(self) -> FixFloat {
        self.unsafe_sqr_length() >> FIX_FRACTION_BITS
    }

    pub fn length(self) -> FixFloat {
        self.unsafe_sqr_length().sqrt()
    }

    pub fn normalize(self) -> Self {
        let l = self.length();
        let s = (1 << 30) / l;
        let x = (s * self.x).sqr_normalize();
        let y = (s * self.y).sqr_normalize();

        Self { x, y }
    }

    pub fn safe_normalize(self) -> Self {
        self.safe_normalize_with_def_value(Self::new_number(0, 1))
    }

    pub fn safe_normalize_with_def_value(self, def: Self) -> Self {
        if self.is_zero() {
            return def;
        }
        self.normalize()
    }

    pub fn half(self) -> FixVec {
        FixVec::new(self.x / 2, self.y / 2)
    }

    pub fn dot_product(self, v: Self) -> FixFloat { // dot product (cos)
        let xx = self.x * v.x;
        let yy = self.y * v.y;
        xx + yy
    }

    pub fn unsafe_dot_product(self, v: Self) -> i64 { // dot product (cos)
        let xx = self.x * v.x;
        let yy = self.y * v.y;
        xx + yy
    }

    pub fn cross_product(self, v: Self) -> FixFloat { // cross product
        let a = self.x * v.y;
        let b = self.y * v.x;

        a - b
    }

    pub fn unsafe_cross_product(self, v: Self) -> i64 { // cross product
        let a = self.x * v.y;
        let b = self.y * v.x;

        a - b
    }

    pub fn cross_product_scalar(self, a: FixFloat) -> Self { // cross product
        let x0 = a * self.y;
        let y0 = a * self.x;

        Self::new(-x0, y0)
    }

    pub fn unsafe_sqr_distance(self, v: Self) -> i64 {
        (self - v).unsafe_sqr_length()
    }

    pub fn sqr_distance(self, v: Self) -> FixFloat {
        (self - v).sqr_length()
    }

    pub fn distance(self, v: Self) -> FixFloat {
        self.sqr_distance(v).sqrt()
    }

    pub fn middle(self, v: Self) -> FixVec {
        let sum = self + v;
        Self::new(sum.x / 2, sum.y / 2)
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}