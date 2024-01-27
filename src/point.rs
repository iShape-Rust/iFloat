use serde::{Deserialize, Serialize};
use crate::fix_vec::FixVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn new_fix_vec(vec: FixVec) -> Self {
        Self { x: vec.x as i32, y: vec.y as i32 }
    }
}