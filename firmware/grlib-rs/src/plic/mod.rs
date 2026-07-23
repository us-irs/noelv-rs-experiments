pub mod regs;

pub use arbitrary_int::u3;
pub use arbitrary_int::traits::Integer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Context {
    Hart0MachineExternal = 0,
    Hart0SupervisorExternal = 1,
    Hart1MachineExternal = 4,
    Hart1SupervisorExternal = 5,
}

pub struct Plic(regs::MmioRegisters<'static>);

#[must_use = "interrupts must be completed or the guard will do it on drop"]
pub struct ClaimedInterrupt<'a> {
    plic: &'a mut Plic,
    context: Context,
    interrupt: usize,
    completed: bool,
}

impl<'a> ClaimedInterrupt<'a> {
    #[inline]
    pub fn interrupt(&self) -> usize {
        self.interrupt
    }

    #[inline]
    pub fn context(&self) -> Context {
        self.context
    }

    #[inline]
    pub fn complete(mut self) {
        if !self.completed {
            self.plic.complete_raw(self.context, self.interrupt);
            self.completed = true;
        }
    }
}

impl Drop for ClaimedInterrupt<'_> {
    #[inline]
    fn drop(&mut self) {
        if !self.completed {
            self.plic.complete_raw(self.context, self.interrupt);
            self.completed = true;
        }
    }
}

impl Plic {
    #[inline]
    pub fn new(regs: regs::MmioRegisters<'static>) -> Self {
        Plic(regs)
    }

    #[inline]
    /// Steal the PLIC register block from a raw base address.
    ///
    /// # Safety
    ///
    /// This is unsafe because it potentially allows creating multiple MMIO handles, which can
    /// lead to data races if the registers are accessed concurrently.
    pub unsafe fn steal(base_addr: usize) -> Self {
        Plic(unsafe { regs::Registers::new_mmio_at(base_addr) })
    }

    #[inline]
    pub fn enable_interrupt(&mut self, context: Context, number: usize) {
        self.set_interrupt_enable(context, number, true);
    }

    #[inline]
    pub fn disable_interrupt(&mut self, context: Context, number: usize) {
        self.set_interrupt_enable(context, number, false);
    }

    #[inline]
    pub fn set_interrupt_priority(&mut self, number: usize, priority: u3) {
        let priority_index = number
            .checked_sub(1)
            .expect("interrupt source 0 has no priority register");
        self.0
            .write_priorities(priority_index, priority.into())
            .unwrap();
    }

    /// If PLIC will mask all interrupts with a priority less than or equal to the threshold.
    ///
    /// Setting 0 will allow all interrupts through, while setting 7 will mask all interrupts.
    #[inline]
    pub fn set_priority_threshold(&mut self, context: Context, threshold: u3) {
        self.0
            .context_prio_and_claim_complete(context as usize)
            .unwrap()
            .write_priority_threshold(threshold.into());
    }

    #[inline]
    pub fn claim(&mut self, context: Context) -> Option<ClaimedInterrupt<'_>> {
        let irq = self
            .0
            .context_prio_and_claim_complete(context as usize)
            .unwrap()
            .read_claim_complete() as usize;
        if irq == 0 {
            None
        } else {
            Some(ClaimedInterrupt {
                plic: self,
                context,
                interrupt: irq,
                completed: false,
            })
        }
    }

    /// Raw interrupt claim function.
    ///
    /// There is a guarded [Self::claim] function which is recommended for general purposes because
    /// it auto-completes on [Drop].
    #[inline]
    pub fn claim_raw(&mut self, context: Context) -> Option<usize> {
        let irq = self
            .0
            .context_prio_and_claim_complete(context as usize)
            .unwrap()
            .read_claim_complete() as usize;
        (irq != 0).then_some(irq)
    }

    /// Raw inteyrrupt complete function.
    ///
    /// If you use [Self::claim_raw] to claim an interrupt, you must call this function to complete
    /// it. If you use [Self::claim], it will automatically complete on [Drop].
    #[inline]
    pub fn complete_raw(&mut self, context: Context, interrupt: usize) {
        self.0
            .context_prio_and_claim_complete(context as usize)
            .unwrap()
            .write_claim_complete(interrupt as u32);
    }

    #[inline(always)]
    pub fn set_interrupt_enable(&mut self, context: Context, number: usize, enable: bool) {
        let reg_offset = number / 32;
        let bit_offset = number % 32;
        self.0
            .context_enable_bits(context as usize)
            .unwrap()
            .modify_enable_bits(reg_offset, |mut val| {
                if enable {
                    val |= 1 << bit_offset;
                } else {
                    val &= !(1 << bit_offset);
                }
                val
            })
            .unwrap();
    }
}

#[cfg(test)]
mod tests {}
