#![no_std]
#![no_main]

use noelv::apb_uart::calculate_scaler;

const SYS_CLK: usize = 40_000_000;
const UART_BAUD: u32 = 115_200;

const COLS: usize = 64;
const ROWS: usize = 21;

#[riscv_rt::entry]
fn main() -> ! {
    let p = noelv::SystemPeripherals::take().expect("failed to take core peripherals");
    let scaler = calculate_scaler(SYS_CLK as u32, UART_BAUD);
    let mut uart = noelv::apb_uart::TxWithShiftRegister::new(p.apb_uart0, Some(scaler));
    uart.write(b"-- NOEL-V Rust Floating Point App --\n\r");

    noelv::log::uart_blocking::init_with_busy_flag(uart, noelv::log::LevelFilter::Trace);

    for r in 0..ROWS {
        // Value represented by this row: +1.0 at the top, -1.0 at the bottom.
        let y = 1.0 - 2.0 * (r as f32) / (ROWS as f32 - 1.0);
        let mut line = [if r == ROWS / 2 { b'-' } else { b' ' }; COLS];

        for (c, cell) in line.iter_mut().enumerate() {
            // Two full periods across the screen.
            let x = 4.0 * core::f32::consts::PI * (c as f32) / (COLS as f32 - 1.0);
            let v = libm::sinf(x);
            // Mark the cell if the curve passes within half a row of it.
            if libm::fabsf(v - y) * (ROWS as f32 - 1.0) * 0.5 < 0.5 {
                *cell = b'*';
            }
        }

        log::info!("{:>5.2} |{}", y, core::str::from_utf8(&line).unwrap());
    }

    log::info!("");
    log::info!("sqrt(2)   = {:.6}", libm::sqrtf(2.0));
    log::info!("exp(1)    = {:.6}", libm::expf(1.0));
    log::info!("0.1 + 0.2 = {:.8}", 0.1f32 + 0.2f32);

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
