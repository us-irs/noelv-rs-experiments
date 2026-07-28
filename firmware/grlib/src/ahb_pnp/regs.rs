//! AMBA Plug&Play configuration record register definitions.
//!
//! Layout taken from the GRLIB IP Core manual (AHBCTRL, "AHB plug&play information record").
pub mod fields {
    use arbitrary_int::{u2, u5, u12};

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Id {
        #[bits(24..=31, r)]
        vendor: u8,
        #[bits(12..=23, r)]
        device: u12,
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

    #[bitbybit::bitenum(u4, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum BarType {
        ApbIo = 0b0001,
        AhbMemory = 0b0010,
        AhbIo = 0b0011,
    }

    #[bitbybit::bitfield(u32, default = 0, debug, forbid_overlaps)]
    pub struct Bar {
        #[bits(20..=31, r)]
        addr: u12,
        #[bit(17, r)]
        prefetchable: bool,
        #[bit(16, r)]
        cacheable: bool,
        #[bits(4..=15, r)]
        mask: u12,
        #[bits(0..=3, r)]
        r#type: Option<BarType>,
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
    pub bar: [fields::Bar; 4],
}

static_assertions::const_assert_eq!(core::mem::size_of::<Registers>(), 32);

impl Registers {
    /// A record is considered unpopulated if all of its words are 0.
    pub fn is_empty(&self) -> bool {
        self.id.raw_value() == 0
            && self.user_defined == [0; 3]
            && self.bar.iter().all(|bar| bar.raw_value() == 0)
    }
}
