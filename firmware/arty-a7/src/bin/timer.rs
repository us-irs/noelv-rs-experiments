#![no_std]
#![no_main]

use core::fmt::Write;

use noelv::{
    apb_uart,
    gp_timer::{self, TimerLowLevel},
    gr_gpio,
    plic::{self, Integer as _},
};

/// The system clock on the Arty-A7 project is configurable and is derived from the board clock
/// which goes through a PLL. Check the `clockers_mig` instantiation in the hardware design
/// for the source of truth. This is not documented inside the datasheets for some reason.
const SYS_CLK: u32 = 40_000_000;
const UART_BAUD: u32 = 115_200;

static CORE_TIMER_INTERRUPT_COUNT: core::sync::atomic::AtomicU32 =
    core::sync::atomic::AtomicU32::new(0);
static GP_TIMER_INTERRUPT_COUNT: core::sync::atomic::AtomicU32 =
    core::sync::atomic::AtomicU32::new(0);

#[riscv_rt::entry]
fn main() -> ! {
    let sys_periphs = noelv::SystemPeripherals::take().expect("failed to take core peripherals");
    let scaler = apb_uart::calculate_scaler(SYS_CLK, UART_BAUD);
    let mut logger = noelv::apb_uart::TxWithShiftRegister::new(sys_periphs.apb_uart0, Some(scaler));
    logger.write(b"-- NOEL-V Timer App --\n\r");
    noelv::log::uart_blocking::init_with_busy_flag(logger, noelv::log::LevelFilter::Trace);

    let (gpio, pins) = gr_gpio::Gpio::new(sys_periphs.gr_gpio);

    // LED pin assignments for the Arty-A7
    let mut led0 = gpio.output_pin(pins.p16, gr_gpio::PinState::Low);
    let mut led1 = gpio.output_pin(pins.p17, gr_gpio::PinState::Low);
    let mut led2 = gpio.output_pin(pins.p18, gr_gpio::PinState::Low);
    let mut led3 = gpio.output_pin(pins.p19, gr_gpio::PinState::Low);

    let mut timer = TimerLowLevel::new(sys_periphs.gp_timer);
    timer
        .0
        .write_scaler_reload(gp_timer::regs::fields::Scaler::new_with_raw_value(4));
    timer
        .0
        .write_scaler(gp_timer::regs::fields::Scaler::new_with_raw_value(4));
    timer
        .0
        .timer_blocks(0)
        .unwrap()
        .write_reload(u16::MAX as u32 * 4);
    timer.modify_config(|val| {
        val.with_enable_timer(0, true)
            .with_enable_set(true)
            .with_enable_latching(false)
    });
    timer
        .modify_timer_control(0, |val| {
            val.with_interrupt_enable(true)
                .with_enable(true)
                .with_restart(true)
                .with_load(true)
        })
        .unwrap();

    noelv::timer::write_compare_value(noelv::HartId::Hart0, u16::MAX as u64);
    // Enable core timer.
    noelv::timer::enable_timer();

    // PLIC handling. The timer interrupt is connected to PLIC interrupt ID 2 and needs to be
    // set up.
    let mut plic = plic::Plic::new(sys_periphs.plic);
    let interrupt_id = noelv::Interrupt::GpTimer1 as usize;
    plic.set_interrupt_priority(interrupt_id, plic::u3::new(1));
    plic.set_priority_threshold(plic::Context::Hart0MachineExternal, plic::u3::ZERO);
    plic.enable_interrupt(plic::Context::Hart0MachineExternal, interrupt_id);

    // Need to enable external interrupts and also enable interrupts globally.
    unsafe {
        noelv::enable_interrupts(true);
        riscv::interrupt::machine::enable_interrupt(riscv::interrupt::Interrupt::MachineTimer);
    }

    let mut delay = noelv::timer::Delay::new(SYS_CLK);
    loop {
        led0.toggle();
        led1.toggle();
        led2.toggle();
        led3.toggle();
        log::info!(
            "Core timer IRQ counter: {}, EXT timer IRQ counter: {}",
            CORE_TIMER_INTERRUPT_COUNT.load(core::sync::atomic::Ordering::Relaxed),
            GP_TIMER_INTERRUPT_COUNT.load(core::sync::atomic::Ordering::Relaxed),
        );

        delay.delay_ms(1000);
    }
}

/// Machine timer interrupt triggers when the compare value is reached.
#[riscv_rt::core_interrupt(riscv::interrupt::Interrupt::MachineTimer)]
fn machine_timer_handler() {
    CORE_TIMER_INTERRUPT_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let timer_value = noelv::timer::read_timer();
    noelv::timer::write_compare_value(noelv::HartId::Hart0, timer_value + u16::MAX as u64);
}

#[riscv_rt::core_interrupt(riscv::interrupt::Interrupt::MachineExternal)]
fn ext_interrupt_handler() {
    let mut plic = plic::Plic::new(unsafe { noelv::SystemPeripherals::steal_plic() });
    let opt_claim_guard = plic.claim(plic::Context::Hart0MachineExternal);
    if let Some(guard) = opt_claim_guard
        && guard.interrupt() == noelv::Interrupt::GpTimer1 as usize
    {
        gptimer_interrupt_handler();
    }
}

fn gptimer_interrupt_handler() {
    GP_TIMER_INTERRUPT_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let mut timer_regs = unsafe { noelv::SystemPeripherals::steal_timer() };
    timer_regs
        .timer_blocks(0)
        .unwrap()
        .modify_control(|val| val.with_interrupt_pending(true).with_interrupt_enable(true));
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let uart = unsafe { noelv::SystemPeripherals::steal_uart() };
    let scaler = apb_uart::calculate_scaler(SYS_CLK, UART_BAUD);
    let mut uart_tx = noelv::apb_uart::TxWithShiftRegister::new(uart, Some(scaler));
    writeln!(&mut uart_tx, "Panic: {}\r", info).unwrap();
    loop {
        riscv::asm::nop();
    }
}
