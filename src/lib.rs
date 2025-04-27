#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod command;
pub mod configuration;
pub mod data;
pub mod error;
mod interface;
mod util;

#[cfg(feature = "async")]
/// Async interface for the SEN66
pub use interface::asynch;

#[cfg(feature = "blocking")]
/// Blocking interface for the SEN66
pub use interface::blocking;
