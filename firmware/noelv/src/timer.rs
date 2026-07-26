pub use crate::HartId;

/// Enable core timer.
#[inline(always)]
pub fn enable_timer() {
    unsafe { riscv::register::mie::set_mtimer() };
}

/// Disable core timer.
#[inline(always)]
pub fn disable_timer() {
    unsafe {
        riscv::register::mie::clear_mtimer();
    };
}

pub fn write_compare_value(hart_id: HartId, value: u64) {
    let mut clint = unsafe { crate::clint::Registers::new_fixed() };
    match hart_id {
        HartId::Hart0 => {
            clint.write_m_timer_compare_low_hart0(value as u32);
            clint.write_m_timer_compare_high_hart0((value >> 32) as u32);
        }
        HartId::Hart1 => {
            clint.write_m_timer_compare_low_hart1(value as u32);
            clint.write_m_timer_compare_high_hart1((value >> 32) as u32);
        }
    }
}

pub fn read_timer() -> u64 {
    let clint = unsafe { crate::clint::Registers::new_fixed() };
    (clint.read_m_timer_high() as u64) << 32 | clint.read_m_timer_low() as u64
}

pub struct Delay {
    sys_clk: u32,
}

impl Delay {
    pub fn new(sys_clk: u32) -> Self {
        Self { sys_clk }
    }

    pub fn delay_ms(&mut self, ms: u32) {
        <Self as embedded_hal::delay::DelayNs>::delay_ms(self, ms)
    }

    pub fn delay_us(&mut self, us: u32) {
        <Self as embedded_hal::delay::DelayNs>::delay_us(self, us)
    }

    pub fn delay_ns(&mut self, ns: u32) {
        if ns == 0 || self.sys_clk == 0 {
            return;
        }

        let ticks = ((ns as u64) * (self.sys_clk as u64 / 2))
            .div_ceil(1_000_000_000)
            .max(1);
        let start = read_timer();

        while read_timer() < start + ticks {
            core::hint::spin_loop();
        }
    }
}

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        self.delay_ns(ns)
    }
}
