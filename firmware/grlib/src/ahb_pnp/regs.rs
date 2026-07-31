//! AMBA Plug&Play configuration record register definitions.
//!
//! Layout taken from the GRLIB IP Core manual (AHBCTRL, "AHB plug&play information record").
pub mod fields {
    use arbitrary_int::{u2, u5, u12};

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Id {
        #[bits(24..=31, r)]
        vendor_raw: u8,
        #[bits(12..=23, r)]
        device_raw: u12,
        // The figure labels both this field and `irq_b` as "IRQ", but does not
        // state how (or whether) they combine into one value - unconfirmed, see
        // the GRLIB IP Library User's Manual for the authoritative description.
        #[bits(10..=11, r)]
        irq_a: u2,
        #[bits(5..=9, r)]
        version: u5,
        #[bits(0..=4, r)]
        irq_b: u5,
    }

    impl Id {
        #[inline]
        pub const fn vendor_id(&self) -> Result<crate::VendorId, u8> {
            crate::VendorId::new_with_raw_value(self.vendor_raw())
        }

        #[inline]
        pub const fn gaisler_device_id(&self) -> Result<crate::GaislerDeviceId, u12> {
            crate::GaislerDeviceId::new_with_raw_value(self.device_raw())
        }
    }

    #[bitbybit::bitenum(u4, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum BarType {
        ApbIo = 0b0001,
        AhbMemory = 0b0010,
        AhbIo = 0b0011,
    }

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct AhbBar {
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

    impl AhbBar {
        #[inline]
        pub const fn address(&self) -> u32 {
            (self.addr_upper_bits().value() as u32) << 20
        }

        /// Size, in bytes, of the address range decoded by this BAR.
        ///
        /// Per the GRLIB IP Core manual, a slave is selected when
        /// `((ADDR xor HADDR(31:20)) and MASK) = 0`. Every `MASK` bit that is 0 turns the
        /// corresponding address bit into a "don't care", doubling the decoded range. Since
        /// only bits 31:20 are compared, the minimum (fully masked, `MASK` all ones) range is
        /// 1 MiB.
        #[inline]
        pub const fn address_range(&self) -> u64 {
            let dont_care_bits = 12 - self.mask().value().count_ones();
            1u64 << (20 + dont_care_bits)
        }
    }
}

/// One 32 byte AMBA Plug&Play configuration record: the identification register followed by
/// 3 user-defined words and 4 bank address registers.
#[derive(derive_mmio::Mmio, Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Registers {
    #[mmio(PureRead)]
    pub id: fields::Id,
    #[mmio(PureRead)]
    pub user_defined: [u32; 3],
    #[mmio(PureRead)]
    pub bar: [fields::AhbBar; 4],
}

static_assertions::const_assert_eq!(core::mem::size_of::<Registers>(), 32);

impl MmioRegisters<'static> {
    /// A record is considered unpopulated if all of its words are 0.
    pub fn is_empty(&self) -> bool {
        self.read_id().raw_value() == 0
            && (0..3).all(|i| self.read_user_defined(i).unwrap() == 0)
            && (0..4).all(|i| self.read_bar(i).unwrap().raw_value() == 0)
    }
}
