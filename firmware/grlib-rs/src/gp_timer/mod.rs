pub mod regs;

pub struct TimerLowLevel(pub regs::MmioRegisters<'static>);

impl TimerLowLevel {
    pub fn new(regs: regs::MmioRegisters<'static>) -> Self {
        Self(regs)
    }

    pub fn modify_config<F: FnOnce(regs::fields::Config) -> regs::fields::Config>(&mut self, f: F) {
        self.0.modify_config(f);
    }

    pub fn modify_timer_control<F: FnOnce(regs::fields::Control) -> regs::fields::Control>(
        &mut self,
        timer_index: usize,
        f: F,
    ) -> Result<(), derive_mmio::OutOfBoundsError> {
        self.0.timer_blocks(timer_index)?.modify_control(f);
        Ok(())
    }
}
