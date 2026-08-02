#![no_std]
extern crate alloc;

#[cfg(feature = "core")]
pub mod adapter;
#[cfg(feature = "core")]
pub mod float;
#[cfg(feature = "core")]
pub mod int;
#[cfg(feature = "core")]
pub mod triangle;

pub mod integration;
