pub const BASE_ADDR: usize = 0xE000_0000;

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    software_interrupt_pending_hart0: u32,
    software_interrupt_pending_hart1: u32,
    _reserved0: [u32; 0xFFE],
    m_timer_compare_low_hart0: u32,
    m_timer_compare_high_hart0: u32,
    m_timer_compare_low_hart1: u32,
    m_timer_compare_high_hart1: u32,
    _reserved1: [u32; 0x1FFA],
    m_timer_low: u32,
    m_timer_high: u32,
}

static_assertions::const_assert_eq!(core::mem::size_of::<Registers>(), 0xC000);

impl Registers {
    /// Create a new MMIO handle to the CLIC registers at the given base address.
    ///
    /// # Safety
    ///
    /// This is unsafe because it potentially allows creating multiple MMIO handles, which can
    /// lead to data races if the registers are accessed concurrently.
    pub unsafe fn new_fixed() -> MmioRegisters<'static> {
        unsafe { Self::new_mmio_at(BASE_ADDR) }
    }
}
