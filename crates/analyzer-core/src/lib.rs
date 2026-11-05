//! Hardware-independent, allocation-free analyzer types and policy.
#![no_std]
#![forbid(unsafe_code)]

pub mod analysis;
pub mod cli;
pub mod config;
pub mod error;
pub mod model;
pub mod state;
