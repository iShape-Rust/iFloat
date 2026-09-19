#![no_std]
#![cfg_attr(feature = "core", doc = include_str!("../README.md"))]
#![cfg_attr(
    not(feature = "core"),
    doc = "Enable the `core` feature (enabled by default) to use the numeric and geometry API."
)]
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
