use std::fmt;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};
use crate::fix_vec::FixVec;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct F32Vec {
    pub x: f32,
    pub y: f32,
}

impl F32Vec {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn to_fix(&self) -> FixVec {
        FixVec::new_f32(self.x, self.y)
    }

    pub fn like_fix(&self) -> FixVec {
        FixVec::new(self.x as i64, self.y as i64)
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn new_i64(x: i64, y: i64) -> Self {
        Self { x: x as f32, y: y as f32 }
    }

    pub fn new_f64(x: f64, y: f64) -> Self {
        Self { x: x as f32, y: y as f32 }
    }

    pub fn dot_product(self, v: Self) -> f64 { // cos
        let xx = (self.x as f64) * (v.x as f64);
        let yy = (self.y as f64) * (v.y as f64);
        xx + yy
    }

    pub fn cross_product(self, v: Self) -> f64 { // sin
        let a = (self.x as f64) * (v.y as f64);
        let b = (self.y as f64) * (v.x as f64);

        a - b
    }

    pub fn sqr_length(&self) -> f64 {
        let x = self.x as f64;
        let y = self.y as f64;
        x * x + y * y
    }

    pub fn length(&self) -> f64 {
        self.sqr_length().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let l = self.length() as f32;
        Self { x: self.x / l, y: self.y / l }
    }
}

impl Mul<f64> for F32Vec {
    type Output = F32Vec;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * (scalar as f32),
            y: self.y * (scalar as f32),
        }
    }
}

impl Add for F32Vec {
    type Output = F32Vec;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for F32Vec {
    type Output = F32Vec;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for F32Vec {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl AddAssign for F32Vec {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl fmt::Display for F32Vec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}
