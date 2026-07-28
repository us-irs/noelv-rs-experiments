pub mod regs;

use regs::Registers;

/// Offset of the plug&play record array from the AHB/APB bridge's own base address: the top
/// 4 kbyte of the bridge's AHB address window (e.g. bridge at 0x80000000 => records at
/// 0x800FF000).
pub const PNP_OFFSET: usize = 0xFF000;
/// Maximum number of APB slaves per bridge (`NAPBSLV` in `amba.vhd`).
pub const RECORDS_PER_BLOCK: usize = 16;

/// Iterates over an AHB/APB bridge's plug&play record array, skipping records where all words
/// are 0 (unused array slots).
pub struct Reader {
    next_addr: usize,
    remaining: usize,
}

impl Reader {
    /// Reader over the plug&play records of the AHB/APB bridge mapped at `bridge_base_addr`.
    ///
    /// # Safety
    ///
    /// [RECORDS_PER_BLOCK] consecutive, readable 8 byte AMBA Plug&Play configuration records
    /// must exist starting at `bridge_base_addr + `[PNP_OFFSET].
    pub const unsafe fn new(bridge_base_addr: usize) -> Self {
        Self {
            next_addr: bridge_base_addr + PNP_OFFSET,
            remaining: RECORDS_PER_BLOCK,
        }
    }

    /// # Safety
    ///
    /// `count` consecutive, readable 8 byte AMBA Plug&Play configuration records must exist
    /// starting at `bridge_base_addr + `[PNP_OFFSET].
    pub const unsafe fn new_with_custom_count(bridge_base_addr: usize, count: usize) -> Self {
        Self {
            next_addr: bridge_base_addr + PNP_OFFSET,
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

            let is_empty = mmio.read_id().raw_value() == 0 && mmio.read_bar().raw_value() == 0;

            if !is_empty {
                return Some(mmio);
            }
        }
        None
    }
}
