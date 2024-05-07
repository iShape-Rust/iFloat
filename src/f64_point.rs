use std::fmt;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct F64Point {
    pub x: f64,
    pub y: f64,
}

impl F64Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline(always)]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn new_i64(x: i64, y: i64) -> Self {
        Self { x: x as f64, y: y as f64 }
    }

    #[inline(always)]
    pub fn dot_product(self, v: Self) -> f64 {
        let xx = self.x * v.x;
        let yy = self.y * v.y;
        xx + yy
    }

    #[inline(always)]
    pub fn cross_product(self, v: Self) -> f64 {
        let a = self.x * v.y;
        let b = self.y * v.x;

        a - b
    }

    #[inline(always)]
    pub fn sqr_length(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    #[inline(always)]
    pub fn length(&self) -> f64 {
        self.sqr_length().sqrt()
    }

    #[inline(always)]
    pub fn normalize(&self) -> Self {
        let l = self.length();
        Self { x: self.x / l, y: self.y / l }
    }
}

impl Mul<f64> for F64Point {
    type Output = F64Point;

    #[inline(always)]
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Add for F64Point {
    type Output = F64Point;

    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for F64Point {
    type Output = F64Point;

    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for F64Point {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl AddAssign for F64Point {

    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl fmt::Display for F64Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}