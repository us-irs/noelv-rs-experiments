#![no_std]
pub use grlib;
pub use grlib::{apb_uart, gr_gpio};

use portable_atomic::AtomicBool;

pub const BASE_ADDR_APBUART0: usize = 0xFC00_1000;
pub const BASE_ADDR_GRGPIO0: usize = 0xFC08_3000;

static PERIPHS_TAKEN: AtomicBool = AtomicBool::new(false);

pub struct CorePeripherals {
    pub apbuart0: apb_uart::regs::MmioRegisters<'static>,
    pub grgpio0: gr_gpio::regs::MmioRegisters<'static>,
}

impl CorePeripherals {
    pub fn take() -> Option<Self> {
        if !PERIPHS_TAKEN.swap(true, portable_atomic::Ordering::Relaxed) {
            return Some(unsafe { Self::steal() });
        }
        None
    }

    pub unsafe fn steal() -> Self {
        Self {
            apbuart0: unsafe { apb_uart::regs::Registers::new_mmio_at(BASE_ADDR_APBUART0) },
            grgpio0: unsafe { gr_gpio::regs::Registers::new_mmio_at(BASE_ADDR_GRGPIO0) },
        }
    }
}

#[cfg(test)]
mod tests {}
