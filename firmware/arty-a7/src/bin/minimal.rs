#![no_std]
#![no_main]

#[riscv_rt::entry]
fn main() -> ! {
    loop {
        riscv::asm::nop();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        riscv::asm::nop();
    }
}
