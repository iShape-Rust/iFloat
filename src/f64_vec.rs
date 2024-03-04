use std::fmt;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};
use crate::fix_vec::FixVec;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct F64Vec {
    pub x: f64,
    pub y: f64,
}

impl F64Vec {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn to_fix(&self) -> FixVec {
        FixVec::new_f64(self.x, self.y)
    }

    pub fn like_fix(&self) -> FixVec {
        FixVec::new(self.x as i64, self.y as i64)
    }

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn new_i64(x: i64, y: i64) -> Self {
        Self { x: x as f64, y: y as f64 }
    }

    pub fn dot_product(self, v: Self) -> f64 { // cos
        let xx = self.x * v.x;
        let yy = self.y * v.y;
        xx + yy
    }

    pub fn cross_product(self, v: Self) -> f64 { // sin
        let a = self.x * v.y;
        let b = self.y * v.x;

        a - b
    }

    pub fn sqr_length(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f64 {
        self.sqr_length().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let l = self.length();
        Self { x: self.x / l, y: self.y / l }
    }
}

impl Mul<f64> for F64Vec {
    type Output = F64Vec;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Add for F64Vec {
    type Output = F64Vec;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for F64Vec {
    type Output = F64Vec;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for F64Vec {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl AddAssign for F64Vec {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl fmt::Display for F64Vec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}