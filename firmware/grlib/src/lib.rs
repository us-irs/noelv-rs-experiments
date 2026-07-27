//! # GRLIB peripheral access library.
//!
//! This crate contains register-level modules for GRLIB peripherals. It provides
//! the low-level building blocks for higher-level abstractions.
#![no_std]
pub mod apb_uart;
pub mod gp_timer;
pub mod gr_gpio;
pub mod plic;

#[cfg(test)]
mod tests {}
