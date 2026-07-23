//! General Purpose Timer (GPTIMER) register definitions.
//!
//! This is a decrementing timer.
pub mod fields {
    use arbitrary_int::{u3, u5};

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Scaler {
        #[bits(0..=15, rw)]
        value: u16,
    }

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Config {
        #[bit(16, rw)]
        enable_timer: [bool; 7],
        #[bit(13, rw)]
        external_event: bool,
        #[bit(12, rw)]
        enable_set: bool,
        #[bit(11, rw)]
        enable_latching: bool,
        #[bit(8, r)]
        separate_interrupts: bool,
        #[bits(3..=7, r)]
        irq: u5,
        #[bits(0..=2, r)]
        num_timers: u3,
    }

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Control {
        #[bits(16..=31, rw)]
        watchdog_increment_reload: u16,
        #[bit(8, rw)]
        disable_watchdog_output: bool,
        #[bit(7, r)]
        enable_watchdog_nmi: bool,
        #[bit(6, r)]
        debug_halt: bool,
        #[bit(5, rw)]
        chain: bool,
        #[bit(4, rw)]
        interrupt_pending: bool,
        #[bit(3, rw)]
        interrupt_enable: bool,
        #[bit(2, rw)]
        load: bool,
        #[bit(1, rw)]
        restart: bool,
        #[bit(0, rw)]
        enable: bool
    }
}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    scaler: fields::Scaler,
    scaler_reload: fields::Scaler,
    config: fields::Config,
    timer_latch_config: u32,

    #[mmio(Inner)]
    timer_blocks: [TimerBlock; 7],
}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct TimerBlock {
    counter: u32,
    reload: u32,
    control: fields::Control,
    latch: u32,
}
