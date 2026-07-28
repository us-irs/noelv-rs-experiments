#![no_std]
#![no_main]

use core::fmt::Write;

use noelv::{ahb_pnp, apb_uart::calculate_scaler, gr_gpio};

/// The system clock on the Arty-A7 project is configurable and is derived from the board clock
/// which goes through a PLL. Check the `clockers_mig` instantiation in the hardware design
/// for the source of truth. This is not documented inside the datasheets for some reason.
const SYS_CLK: usize = 40_000_000;
const UART_BAUD: u32 = 115_200;

#[riscv_rt::entry]
fn main() -> ! {
    let core = noelv::SystemPeripherals::take().expect("failed to take core peripherals");
    let scaler = calculate_scaler(SYS_CLK as u32, UART_BAUD);
    let mut logger = noelv::apb_uart::TxWithShiftRegister::new(core.apb_uart0, Some(scaler));
    writeln!(&mut logger, "-- NOEL-V Rust Sample App --\r").unwrap();

    let (gpio, pins) = gr_gpio::Gpio::new(core.gr_gpio);

    // LED pin assignments for the Arty-A7
    let mut led0 = gpio.output_pin(pins.p16, gr_gpio::PinState::Low);

    let reader = unsafe { ahb_pnp::Reader::new_for_masters() };
    for (index, ahb_block) in reader.enumerate() {
        log::info!(
            "AHB Plug&Play Block {} at address {}",
            index,
            unsafe { ahb_block.ptr() } as u32
        );
        log::info!("ID: {:?}", ahb_block.read_id());
    }
    loop {
        led0.toggle();
        for _ in 0..1000000 {
            riscv::asm::nop();
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        riscv::asm::nop();
    }
}
