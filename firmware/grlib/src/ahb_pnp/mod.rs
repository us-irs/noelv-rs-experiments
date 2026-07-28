pub mod regs;

use regs::Registers;

/// Iterates over consecutive 32 byte AMBA Plug&Play configuration records starting at a given
/// address, skipping records where all words are 0 (unused array slots).
pub struct Reader {
    next_addr: usize,
    remaining: usize,
}

/// Default AHB master block address (`cfgaddr`/`ioaddr` reset value).
pub const DEFAULT_MASTER_BASE_ADDR: usize = 0xFFFF_F000;
/// Default AHB slave block address: the second 2 kbyte half of the default configuration area.
pub const DEFAULT_SLAVE_BASE_ADDR: usize = 0xFFFF_F800;
/// Number of 32 byte records in one 2 kbyte AMBA Plug&Play block (masters or slaves).
pub const RECORDS_PER_BLOCK: usize = 2048 / core::mem::size_of::<Registers>();

impl Reader {
    /// Reader over the AHB master block at [DEFAULT_MASTER_BASE_ADDR].
    ///
    /// # Safety
    ///
    /// [RECORDS_PER_BLOCK] consecutive, readable 32 byte AMBA Plug&Play configuration records
    /// must exist starting at [DEFAULT_MASTER_BASE_ADDR].
    pub const unsafe fn new_for_masters() -> Self {
        unsafe { Self::new(DEFAULT_MASTER_BASE_ADDR) }
    }

    /// Reader over the AHB slave block at [DEFAULT_SLAVE_BASE_ADDR].
    ///
    /// # Safety
    ///
    /// [RECORDS_PER_BLOCK] consecutive, readable 32 byte AMBA Plug&Play configuration records
    /// must exist starting at [DEFAULT_SLAVE_BASE_ADDR].
    pub const unsafe fn new_for_slaves() -> Self {
        unsafe { Self::new(DEFAULT_SLAVE_BASE_ADDR) }
    }

    /// # Safety
    ///
    /// `base_addr` must point to [RECORDS_PER_BLOCK] consecutive, readable 32 byte AMBA
    /// Plug&Play configuration records.
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self {
            next_addr: base_addr,
            remaining: RECORDS_PER_BLOCK,
        }
    }

    pub const unsafe fn new_with_custom_count(base_addr: usize, count: usize) -> Self {
        Self {
            next_addr: base_addr,
            remaining: count,
        }
    }
}

impl Iterator for Reader {
    type Item = regs::MmioRegisters<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.remaining > 0 {
            self.remaining -= 1;
            let addr = self.next_addr;
            self.next_addr += core::mem::size_of::<Registers>();

            let mmio = unsafe { Registers::new_mmio_at(addr) };

            let is_empty = mmio.read_id().raw_value() == 0
                && (0..3).all(|i| mmio.read_user_defined(i).unwrap() == 0)
                && (0..4).all(|i| mmio.read_bar(i).unwrap().raw_value() == 0);

            if !is_empty {
                return Some(mmio);
            }
        }
        None
    }
}
