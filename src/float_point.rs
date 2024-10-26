use std::fmt;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};
use crate::float::Float;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FloatPoint<T: Float> {
    pub x: T,
    pub y: T,
}

impl<T: Float> FloatPoint<T> {
    #[inline(always)]
    pub fn zero() -> Self {
        Self { x: Float::zero(), y: Float::zero() }
    }

    #[inline(always)]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn dot_product(self, v: Self) -> T {
        let xx: T = self.x * v.x;
        let yy: T = self.y * v.y;
        xx + yy
    }

    #[inline(always)]
    pub fn cross_product(self, v: Self) -> T {
        let a = self.x * v.y;
        let b = self.y * v.x;

        a - b
    }

    #[inline(always)]
    pub fn sqr_length(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    #[inline(always)]
    pub fn length(&self) -> T {
        self.sqr_length().sqrt()
    }

    #[inline(always)]
    pub fn normalize(&self) -> Self {
        let l = self.length();
        Self { x: self.x / l, y: self.y / l }
    }
}

impl<T: Float> Mul<T> for FloatPoint<T> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T: Float> Add for FloatPoint<T> {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Float> Sub for FloatPoint<T> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Float> Neg for FloatPoint<T> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl<T: Float> AddAssign for FloatPoint<T> {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.x = self.x + other.x;
        self.y = self.y + other.y;
    }
}

impl<T: Float> fmt::Display for FloatPoint<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

pub trait FloatPointCompatible<T: Float> {
    fn from_float_point(float_point: FloatPoint<T>) -> Self;
    fn to_float_point(&self) -> FloatPoint<T>;
}

impl<T: Float> FloatPointCompatible<T> for FloatPoint<T> {
    #[inline(always)]
    fn from_float_point(float_point: FloatPoint<T>) -> Self {
        float_point
    }

    #[inline(always)]
    fn to_float_point(&self) -> FloatPoint<T> {
        *self
    }
}