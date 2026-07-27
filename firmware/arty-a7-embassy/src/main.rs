#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use noelv::{apb_uart::calculate_scaler, gr_gpio};

/// The system clock on the Arty-A7 project is configurable and is derived from the board clock
/// which goes through a PLL. Check the `clockers_mig` instantiation in the hardware design
/// for the source of truth. This is not documented inside the datasheets for some reason.
const SYS_CLK: usize = 40_000_000;
const UART_BAUD: u32 = 115_200;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let core = noelv::SystemPeripherals::take().expect("failed to take core peripherals");
    let scaler = calculate_scaler(SYS_CLK as u32, UART_BAUD);
    // Need to initialize the timer driver.
    noelv::time_driver_mtimer::init(SYS_CLK as u32, true);

    let mut logger = noelv::apb_uart::TxWithShiftRegister::new(core.apb_uart0, Some(scaler));
    writeln!(&mut logger, "-- NOEL-V Embassy Example App --\r").unwrap();

    let (gpio, pins) = gr_gpio::Gpio::new(core.gr_gpio);

    // LED pin assignments for the Arty-A7
    let mut led0 = gpio.output_pin(pins.p16, gr_gpio::PinState::Low);
    let mut led1 = gpio.output_pin(pins.p17, gr_gpio::PinState::Low);
    let mut led2 = gpio.output_pin(pins.p18, gr_gpio::PinState::Low);
    let mut led3 = gpio.output_pin(pins.p19, gr_gpio::PinState::Low);

    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        led0.toggle();
        led1.toggle();
        led2.toggle();
        led3.toggle();
        ticker.next().await;
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        riscv::asm::nop();
    }
}
