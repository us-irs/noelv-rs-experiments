#![no_std]
#![no_main]

use core::fmt::Write;

use noelv::{ahb_pnp, apb_uart::calculate_scaler, gr_gpio, uprintln};

/// The system clock on the Arty-A7 project is configurable and is derived from the board clock
/// which goes through a PLL. Check the `clockers_mig` instantiation in the hardware design
/// for the source of truth. This is not documented inside the datasheets for some reason.
const SYS_CLK: usize = 40_000_000;
const UART_BAUD: u32 = 115_200;
const PRINT_ADDR_RANGE: bool = false;

/// Prints every entry of the AHB master Plug&Play array.
fn print_masters() {
    uprintln!("=== AHB masters ===");
    let master_reader = unsafe { ahb_pnp::Reader::new_for_masters() };
    for ahb_block in master_reader {
        let dev_id = ahb_block.read_id().gaisler_device_id();
        uprintln!(
            "AHB master {:?} at address {:#010x}",
            dev_id,
            unsafe { ahb_block.ptr() } as u32
        );
    }
}

/// Prints every entry of the AHB slave Plug&Play array, descending into the APB Plug&Play array
/// of any AHB/APB bridge found along the way.
fn print_slaves() {
    uprintln!("=== AHB slaves ===");
    let slave_reader = unsafe { ahb_pnp::Reader::new_for_slaves() };
    for ahb_block in slave_reader {
        let dev_id = ahb_block.read_id().gaisler_device_id();
        uprintln!(
            "AHB slave {:?} at address {:#010x}",
            dev_id,
            unsafe { ahb_block.ptr() } as u32
        );
        let bar = ahb_block.read_bar(0).unwrap();
        if PRINT_ADDR_RANGE {
            uprintln!("Address range: {}", bar.address_range());
        }
        if dev_id == Ok(noelv::grlib::GaislerDeviceId::ApbMst) {
            print_apb_devices(bar.address());
        }
    }
}

/// Prints every entry of the APB Plug&Play array belonging to the AHB/APB bridge mapped at
/// `bridge_base_addr`.
fn print_apb_devices(bridge_base_addr: u32) {
    uprintln!("    === APB master address {:#010x} ===", bridge_base_addr);
    let apb_reader = unsafe { noelv::apb_pnp::Reader::new(bridge_base_addr as usize) };
    for apb_block in apb_reader {
        let bar = apb_block.read_bar();
        let dev_id = apb_block.read_id().gaisler_device_id();
        uprintln!(
            "    APB Device ID {:?} at address: {:#010x}",
            dev_id,
            bar.total_address(bridge_base_addr)
        );
        if PRINT_ADDR_RANGE {
            uprintln!("   Address range: {}", bar.address_range());
        }
    }
}

/// Blinks `led` forever, once per busy-loop spin.
fn blink_forever(mut led: gr_gpio::Output) -> ! {
    loop {
        led.toggle();
        for _ in 0..1000000 {
            riscv::asm::nop();
        }
    }
}

#[riscv_rt::entry]
fn main() -> ! {
    let core = noelv::SystemPeripherals::take().expect("failed to take core peripherals");
    let scaler = calculate_scaler(SYS_CLK as u32, UART_BAUD);
    let mut logger = noelv::apb_uart::TxWithShiftRegister::new(core.apb_uart0, Some(scaler));
    writeln!(&mut logger, "-- NOEL-V AHB Scanning App --\r").unwrap();

    noelv::log::uart_blocking::init_with_busy_flag(logger, noelv::log::LevelFilter::Trace);

    let (gpio, pins) = gr_gpio::Gpio::new(core.gr_gpio);

    // LED pin assignments for the Arty-A7
    let led0 = gpio.output_pin(pins.p16, gr_gpio::PinState::Low);

    print_masters();
    uprintln!("");
    print_slaves();

    blink_forever(led0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        riscv::asm::nop();
    }
}
