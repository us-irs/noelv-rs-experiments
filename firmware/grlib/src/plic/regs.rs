#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct ContextEnableBitRegs {
    enable_bits: [u32; 0x20],
}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct ContextPrioAndClaimCompleteRegs {
    priority_threshold: u32,
    claim_complete: u32,
    _reserved: [u32; 0x3fe],
}

/// PLIC register block definition.
#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    _reserved: u32,
    priorities: [u32; 1023],
    pending_bits: [u32; 0x20],
    _reserved2: [u32; 0x3e0],
    #[mmio(Inner)]
    context_enable_bits: [ContextEnableBitRegs; 0x8],
    _reserved3: [u32; 0x7F700],
    #[mmio(Inner)]
    context_prio_and_claim_complete: [ContextPrioAndClaimCompleteRegs; 0x8],
    _reserved4: [u32; 0xF7E000],
}

static_assertions::const_assert_eq!(core::mem::size_of::<Registers>(), 0x4000000);
