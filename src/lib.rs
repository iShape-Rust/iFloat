#![no_std]
extern crate alloc;

#[cfg(feature = "core")]
pub mod fix_angle;
#[cfg(feature = "core")]
pub mod fix_float;
#[cfg(feature = "core")]
pub mod fix_vec;
#[cfg(feature = "core")]
pub mod triangle;
#[cfg(feature = "core")]
pub mod u128;
#[cfg(feature = "core")]
pub mod int;
#[cfg(feature = "core")]
pub mod adapter;
#[cfg(feature = "core")]
mod fix_sin;

#[cfg(any(feature = "float_pt", feature = "core"))]
pub mod float;

pub mod integration;