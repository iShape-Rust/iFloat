use std::cmp::Ordering;
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::fix_vec::FixVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IntPoint {
    pub x: i32,
    pub y: i32,
}

impl IntPoint {

    pub const ZERO: Self = Self { x: 0, y: 0 };

    #[inline(always)]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn new_fix_vec(vec: FixVec) -> Self {
        Self { x: vec.x as i32, y: vec.y as i32 }
    }

    #[inline(always)]
    pub fn cross_product(self, v: Self) -> i64 {
        let a = (self.x as i64) * (v.y as i64);
        let b = (self.y as i64) * (v.x as i64);

        a - b
    }

    #[inline(always)]
    pub fn dot_product(self, v: Self) -> i64 {
        let xx = (self.x as i64) * (v.x as i64);
        let yy = (self.y as i64) * (v.y as i64);
        xx + yy
    }

    #[inline(always)]
    pub fn subtract(self, other: IntPoint) -> FixVec {
        let x = (self.x as i64) - (other.x as i64);
        let y = (self.y as i64) - (other.y as i64);
        FixVec::new(x, y)
    }

    #[inline(always)]
    pub fn sqr_distance(self, other: IntPoint) -> i64 {
        let x = (self.x as i64) - (other.x as i64);
        let y = (self.y as i64) - (other.y as i64);
        return x * x + y * y;
    }
}

impl fmt::Display for IntPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl PartialOrd for IntPoint {

    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntPoint {

    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let x = self.x == other.x;
        if x && self.y == other.y {
            return Ordering::Equal
        } else if self.x < other.x || x && self.y < other.y {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}