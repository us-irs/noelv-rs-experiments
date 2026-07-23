#![no_std]
pub use grlib;
pub use grlib::{apb_uart, gp_timer, gr_gpio, plic};

pub mod clint;
pub mod timer;

use portable_atomic::AtomicBool;

pub const BASE_ADDR_APBUART0: usize = 0xFF90_0000;
pub const BASE_ADDR_GRGPIO0: usize = 0xFF98_3000;
pub const BASE_ADDR_GPTIMER0: usize = 0xFF90_8000;
pub const BASE_ADDR_PLIC: usize = 0xF800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HartId {
    Hart0 = 0,
    Hart1 = 1,
}

static PERIPHS_TAKEN: AtomicBool = AtomicBool::new(false);

/// Enable interrupts for the NOEL-V core. This function enables the machine external interrupt
/// (MEXT) as well if specified.
///
/// # Safety
///
/// Enabling interrupts might break critical sections or other synchronization mechanisms.
/// Ensure that this is called in a safe context where interrupts can be enabled.
pub unsafe fn enable_interrupts(external: bool) {
    unsafe {
        if external {
            riscv::register::mie::set_mext();
        }
        riscv::interrupt::enable();
    }
}

pub struct SystemPeripherals {
    pub apb_uart0: apb_uart::regs::MmioRegisters<'static>,
    pub gr_gpio: gr_gpio::regs::MmioRegisters<'static>,
    /// This is the external timer provided as part of the subsystem. This is not the machine/core
    /// timer.
    pub gp_timer: gp_timer::regs::MmioRegisters<'static>,
    /// Platform-level interrupt controller (PLIC) for the NOEL-V.
    pub plic: plic::regs::MmioRegisters<'static>,
    /// Core-local interrupt controller (CLINT) for the NOEL-V.
    pub clint: clint::MmioRegisters<'static>,
}

impl SystemPeripherals {
    pub fn take() -> Option<Self> {
        if !PERIPHS_TAKEN.swap(true, portable_atomic::Ordering::Relaxed) {
            return Some(unsafe { Self::steal() });
        }
        None
    }

    /// Steal all core peripheral register blocks.
    ///
    /// # Safety
    ///
    /// This is unsafe because it potentially allows creating multiple MMIO handles, which can
    /// lead to data races if the registers are accessed concurrently.
    pub unsafe fn steal() -> Self {
        Self {
            apb_uart0: unsafe { Self::steal_uart() },
            gr_gpio: unsafe { gr_gpio::regs::Registers::new_mmio_at(BASE_ADDR_GRGPIO0) },
            gp_timer: unsafe { Self::steal_timer() },
            plic: unsafe { Self::steal_plic() },
            clint: unsafe { clint::Registers::new_fixed() },
        }
    }

    /// Steal the core logger UART (APBUART0) registers.
    ///
    /// # Safety
    ///
    /// This is unsafe because it potentially allows creating multiple MMIO handles, which can
    /// lead to data races if the registers are accessed concurrently.
    pub unsafe fn steal_uart() -> apb_uart::regs::MmioRegisters<'static> {
        unsafe { apb_uart::regs::Registers::new_mmio_at(BASE_ADDR_APBUART0) }
    }

    /// Steal the core timer (GPTIMER) registers.
    ///
    /// # Safety
    ///
    /// This is unsafe because it potentially allows creating multiple MMIO handles, which can
    /// lead to data races if the registers are accessed concurrently.
    pub unsafe fn steal_timer() -> gp_timer::regs::MmioRegisters<'static> {
        unsafe { gp_timer::regs::Registers::new_mmio_at(BASE_ADDR_GPTIMER0) }
    }

    /// Steal the platform-level interrupt controller (PLIC) registers.
    ///
    /// # Safety
    ///
    /// This is unsafe because it potentially allows creating multiple MMIO handles, which can
    /// lead to data races if the registers are accessed concurrently.
    pub unsafe fn steal_plic() -> plic::regs::MmioRegisters<'static> {
        unsafe { plic::regs::Registers::new_mmio_at(BASE_ADDR_PLIC) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    ApbUart = 1,
    GpTimer1 = 2,
    GpTimer2 = 3,
    AhbState = 4,
    GrethGbit = 5,
}

unsafe impl riscv_rt::InterruptNumber for Interrupt {
    const MAX_INTERRUPT_NUMBER: usize = 5;

    fn number(self) -> usize {
        self as usize
    }

    fn from_number(value: usize) -> riscv_rt::result::Result<Self> {
        match value {
            1 => Ok(Interrupt::ApbUart),
            2 => Ok(Interrupt::GpTimer1),
            3 => Ok(Interrupt::GpTimer2),
            4 => Ok(Interrupt::AhbState),
            5 => Ok(Interrupt::GrethGbit),
            _ => Err(riscv_rt::result::Error::InvalidVariant(value)),
        }
    }
}

#[cfg(test)]
mod tests {}
