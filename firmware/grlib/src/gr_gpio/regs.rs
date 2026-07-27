pub mod fields {}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    data: u32,
    output: u32,
    dir: u32,
    interrupt_mask: u32,
    interrupt_polarity: u32,
    interrupt_edge: u32,
    bypass: u32,
    capability: u32,

    interrupt_map: [u32; 0x8],

    interrupt_available: u32,
    interrupt_flag: u32,
    input_enable: u32,
    pulse: u32,

    input_enable_logic_or: u32,
    output_logic_or: u32,
    dir_logic_or: u32,
    interrupt_mask_logic_or: u32,

    input_enable_logic_and: u32,
    output_logic_and: u32,
    dir_logic_and: u32,
    interrupt_mask_logic_and: u32,

    input_enable_logic_xor: u32,
    output_logic_xor: u32,
    dir_logic_xor: u32,
    interrupt_mask_logic_xor: u32,
}

static_assertions::const_assert_eq!(core::mem::size_of::<Registers>(), 0x80);
