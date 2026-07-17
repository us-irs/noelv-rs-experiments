
pub mod fields {
    use arbitrary_int::{u4, u6};

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Data {
        #[bits(0..=7 ,rw)]
        data: u8,
    }

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Status {
        #[bits(26..=31, rw)]
        rx_fifo_count: u6,
        #[bits(20..=25, rw)]
        tx_fifo_count: u6,
        #[bit(10, rw)]
        rx_fifo_full: bool,
        #[bit(9, rw)]
        tx_fifo_full: bool,
        #[bit(8, rw)]
        rx_fifo_half_full: bool,
        #[bit(7, rw)]
        tx_fifo_half_full: bool,
        #[bit(6, rw)]
        framing_error: bool,
        #[bit(5, rw)]
        parity_error: bool,
        #[bit(4, rw)]
        overrun: bool,
        #[bit(3, rw)]
        r#break: bool,
        #[bit(2, rw)]
        tx_fifo_empty: bool,
        #[bit(1, rw)]
        tx_shiftreg_empty: bool,
        #[bit(0, rw)]
        data_ready: bool,
    }

    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum ParitySelect {
        Even = 0,
        Odd = 1,
    }

    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum StopBits {
        One = 0,
        Two = 1,
    }

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Control {
        #[bit(31, r)]
        fifo_available: bool,
        #[bit(20, rw)]
        transmit_break: bool,
        #[bits(16..=19, rw)]
        break_size: u4,
        #[bit(15, rw)]
        stop_bits: StopBits,
        #[bit(14, rw)]
        tx_shiftreg_empty_interrupt_enable: bool,
        #[bit(13, rw)]
        delayed_interrupt_enable: bool,
        #[bit(12, rw)]
        break_interrupt_enable: bool,
        #[bit(11, r)]
        fifo_debug_enabled: bool,
        #[bit(10, rw)]
        rx_fifo_interrupt_enable: bool,
        #[bit(9, rw)]
        tx_fifo_interrupt_enable: bool,
        #[bit(8, rw)]
        external_clock: bool,
        #[bit(7, rw)]
        loopback: bool,
        #[bit(6, rw)]
        flow_control: bool,
        #[bit(5, rw)]
        parity_enable: bool,
        #[bit(4, rw)]
        parity_select: ParitySelect,
        #[bit(3, rw)]
        enable_tx_interrupt: bool,
        #[bit(2, rw)]
        enable_rx_interrupt: bool,
        #[bit(1, rw)]
        enable_tx: bool,
        #[bit(0, rw)]
        enable_rx: bool,
    }

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Capability {
        #[bit(6, rw)]
        flow_control: bool,
        #[bits(0..=5, rw)]
        fifo_size: u6,
    }

}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    data: fields::Data,
    status: fields::Status,
    control: fields::Control,
    scaler: u32,
    fifo_debug: fields::Data,
    fifo_debug_control: u32,
    capability: fields::Capability,
}
