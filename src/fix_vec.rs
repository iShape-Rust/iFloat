use std::ops;
use crate::fix_float;
use crate::fix_float::FixFloat;

#[derive(Debug, Clone, Copy)]
pub struct FixVec {
    pub x: FixFloat,
    pub y: FixFloat,
}

impl FixVec {
    pub const ZERO: Self = Self { x: FixFloat::ZERO, y: FixFloat::ZERO };

    pub fn is_zero(&self) -> bool {
        self.x == FixFloat::ZERO && self.y == FixFloat::ZERO
    }

    pub fn bit_pack(&self) -> i64 {
        (self.x.value() << FixFloat::MAX_BITS) + self.y.value()
    }

    pub fn new_i64(x: i64, y: i64) -> Self {
        Self { x: FixFloat::new_i64(x), y: FixFloat::new_i64(y) }
    }

    pub fn new_number(x: i64, y: i64) -> Self {
        Self { x: FixFloat::new_number(x), y: FixFloat::new_number(y) }
    }

    pub fn new_fix(x: FixFloat, y: FixFloat) -> Self {
        Self { x, y }
    }

    pub fn sqr_length(&self) -> FixFloat {
        let x = self.x.value();
        let y = self.y.value();
        FixFloat::new_i64((x * x + y * y) >> FixFloat::FRACTION_BITS)
    }

    pub fn length(&self) -> FixFloat {
        let x = self.x.value();
        let y = self.y.value();
        let sqr = x * x + y * y;
        let sqrt = fix_float::fast_square_root(sqr);
        FixFloat::new_i64(sqrt)
    }

    pub fn normalize(&self) -> Self {
        let l = self.length();
        let s = (1 << 30) / l.value();
        let xx = fix_float::sqr_normalize(s * self.x.value());
        let yy = fix_float::sqr_normalize(s * self.y.value());

        Self::new_fix(xx, yy)
    }

    pub fn safe_normalize(&self) -> Self {
        self.safe_normalize_with_def_value(Self::new_number(0,1))
    }

    pub fn safe_normalize_with_def_value(&self, def: Self) -> Self {
        if self.is_zero() {
            return def;
        }
        self.normalize()
    }

    pub fn half(&self) -> FixVec {
        FixVec::new_i64(self.x.value() / 2, self.y.value() / 2)
    }

    pub fn dot_product(&self, v: &Self) -> FixFloat { // dot product (cos)
        let xx = self.x * v.x;
        let yy = self.y * v.y;
        xx + yy
    }

    pub fn unsafe_dot_product(&self, v: &Self) -> i64 { // dot product (cos)
        let xx = self.x.value() * v.x.value();
        let yy = self.y.value() * v.y.value();
        xx + yy
    }

    pub fn cross_product(&self, v: &Self) -> FixFloat { // cross product
        let a = self.x * v.y;
        let b = self.y * v.x;

        a - b
    }
    
    pub fn unsafe_cross_product(&self, v: &Self) -> i64 { // cross product
        let a = self.x.value() * v.y.value();
        let b = self.y.value() * v.x.value();

        a - b
    }

    pub fn cross_product_scalar(&self, a: FixFloat) -> Self { // cross product
        let x0 = a * self.y;
        let y0 = a * self.x;

        FixVec::new_fix(-x0, y0)
    }

    pub fn sqr_distance(&self, v: &Self) -> FixFloat {
        let a = *self - *v;
        a.sqr_length()
    }

    pub fn distance(&self, v: &Self) -> FixFloat {
        self.sqr_distance(v).sqrt()
    }

    pub fn middle(&self, v: &Self) -> FixVec {
        let sum = *self + *v;
        Self::new_i64(sum.x.value() / 2, sum.y.value() / 2)
    }
    
}


impl ops::Add for FixVec {
    type Output = FixVec;

    fn add(self, other: FixVec) -> FixVec {
        FixVec {
            x: self.x + other.x,
            y: self.y + other.y
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

impl PartialEq for FixVec {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}


impl Eq for FixVec {}