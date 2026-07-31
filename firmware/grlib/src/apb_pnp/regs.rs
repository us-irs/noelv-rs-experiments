//! APB AMBA Plug&Play configuration record register definitions.
//!
//! Layout taken from the GRLIB IP Core manual (APBCTRL, "APB plug&play information"). The
//! identification word is bit-for-bit identical to the AHB one (see [crate::ahb_pnp::regs]), so
//! it is reused here directly. Unlike the AHB record, there is only a single bank address
//! register, and its `C/P` nibble is always zero (see `apb_iobar` in `amba.vhd`) rather than
//! carrying independent cacheable/prefetchable bits.

pub mod fields {
    pub use crate::ahb_pnp::regs::fields::Id;
    use arbitrary_int::u12;

    use crate::ahb_pnp::regs::fields::BarType;

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct ApbBar {
        #[bits(20..=31, r)]
        addr_upper_bits: u12,
        #[bit(17, r)]
        prefetchable: bool,
        #[bit(16, r)]
        cacheable: bool,
        #[bits(4..=15, r)]
        mask: u12,
        #[bits(0..=3, r)]
        bar_type: Option<BarType>,
    }

    impl ApbBar {
        /// HADDR(19:8) are decoded, so we need to shift the address field 9 to the left
        /// and add that to the base address of the APB controller.
        #[inline]
        pub const fn address(&self) -> u32 {
            (self.addr_upper_bits().value() as u32) << 8
        }

        #[inline]
        pub const fn total_address(&self, ahb_base: u32) -> u32 {
            ahb_base + self.address()
        }

        /// Size, in bytes, of the address range decoded by this BAR.
        ///
        /// Mirrors [`crate::ahb_pnp::regs::fields::AhbBar::size`], but an APB I/O BAR decodes
        /// `PADDR(19:8)` instead of `HADDR(31:20)`, so the minimum (fully masked) range is
        /// 256 bytes instead of 1 MiB.
        #[inline]
        pub const fn address_range(&self) -> u32 {
            let dont_care_bits = 12 - self.mask().value().count_ones();
            1u32 << (8 + dont_care_bits)
        }
    }
}

/// One 8 byte APB AMBA Plug&Play configuration record: the identification register followed by
/// a single bank address register.
#[derive(derive_mmio::Mmio, Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Registers {
    #[mmio(PureRead)]
    pub id: fields::Id,
    #[mmio(PureRead)]
    pub bar: fields::ApbBar,
}

static_assertions::const_assert_eq!(core::mem::size_of::<Registers>(), 8);

impl MmioRegisters<'static> {
    /// A record is considered unpopulated if all of its words are 0.
    pub fn is_empty(&self) -> bool {
        self.read_id().raw_value() == 0 && self.read_bar().raw_value() == 0
    }
}
