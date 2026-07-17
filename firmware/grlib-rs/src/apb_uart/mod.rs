pub mod regs;

pub fn calculate_scaler(sys_clk: u32, baud: u32) -> u32 {
    sys_clk / (baud * 8) - 1
}

pub struct TxWithShiftRegister(regs::MmioRegisters<'static>);

impl TxWithShiftRegister {
    pub fn new(mut regs: regs::MmioRegisters<'static>, clk_scaler: Option<u32>) -> Self {
        regs.modify_control(|val| val.with_enable_tx(true));
        if let Some(clk_scaler) = clk_scaler {
            regs.write_scaler(clk_scaler);
        }
        Self(regs)
    }

    #[inline(always)]
    pub fn write_byte_unchecked(&mut self, data: u8) {
        self.0.write_data(regs::fields::Data::ZERO.with_data(data));
    }

    pub fn write_byte(&mut self, data: u8) {
        while self.0.read_status().tx_fifo_full() {}
        self.write_byte_unchecked(data);
    }

    pub fn write(&mut self, buf: &[u8]) {
        for &b in buf {
            self.write_byte(b);
        }
    }
}

impl core::fmt::Write for TxWithShiftRegister {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}
