use serde::{Deserialize, Serialize};
use crate::fix_vec::FixVec;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct F32Vec {
    pub x: f32,
    pub y: f32,
}

impl F32Vec {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn fix(&self) -> FixVec {
        FixVec::new_f32(self.x, self.y)
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn new_f64(x: f64, y: f64) -> Self {
        Self { x: x as f32, y: y as f32 }
    }
}
