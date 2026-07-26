//! # Simple logging providers

use core::sync::atomic::{AtomicBool, AtomicU8};

pub use log::LevelFilter;

static LOGGER_INIT_DONE: AtomicBool = AtomicBool::new(false);

const LOG_SEL_LOCKED: u8 = 1;
const LOG_SEL_UNSAFE_SINGLE_CORE: u8 = 2;

static LOG_SEL: AtomicU8 = AtomicU8::new(0);

/// Blocking UART loggers.
pub mod uart_blocking {
    use super::*;
    use core::cell::{RefCell, UnsafeCell};
    use embedded_io::Write as _;

    use critical_section::Mutex;
    use log::{LevelFilter, Log, set_logger, set_max_level};

    use crate::grlib::apb_uart::TxWithShiftRegister;

    pub struct UartLoggerBlocking(Mutex<RefCell<Option<TxWithShiftRegister>>>);

    unsafe impl Send for UartLoggerBlocking {}
    unsafe impl Sync for UartLoggerBlocking {}

    static UART_LOGGER_BLOCKING: UartLoggerBlocking =
        UartLoggerBlocking(Mutex::new(RefCell::new(None)));

    /// Initialize the logger with a blocking UART instance.
    ///
    /// This is a blocking logger which performs a write inside a critical section. This logger is
    /// thread-safe, but interrupts will be disabled while the logger is writing to the UART.
    ///
    /// For async applications, it is strongly recommended to use the asynchronous ring buffer
    /// logger instead.
    pub fn init_with_locks(uart: TxWithShiftRegister, level: LevelFilter) {
        if LOGGER_INIT_DONE.swap(true, core::sync::atomic::Ordering::Relaxed) {
            return;
        }
        LOG_SEL.swap(LOG_SEL_LOCKED, core::sync::atomic::Ordering::Relaxed);
        critical_section::with(|cs| {
            let inner = UART_LOGGER_BLOCKING.0.borrow(cs);
            inner.replace(Some(uart));
        });
        set_logger(&UART_LOGGER_BLOCKING).unwrap();
        // Adjust as needed
        set_max_level(level);
    }

    impl log::Log for UartLoggerBlocking {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            critical_section::with(|cs| {
                let mut opt_logger = self.0.borrow(cs).borrow_mut();
                if opt_logger.is_none() {
                    return;
                }
                let logger = opt_logger.as_mut().unwrap();
                writeln!(logger, "{} - {}\r", record.level(), record.args()).unwrap();
            })
        }

        fn flush(&self) {
            critical_section::with(|cs| {
                let mut opt_logger = self.0.borrow(cs).borrow_mut();
                if opt_logger.is_none() {
                    return;
                }
                let logger = opt_logger.as_mut().unwrap();
                logger.flush().unwrap();
            });
        }
    }

    pub struct UartLoggerWithBusyFlag {
        busy: AtomicBool,
        uart: UnsafeCell<Option<TxWithShiftRegister>>,
    }

    unsafe impl Send for UartLoggerWithBusyFlag {}
    unsafe impl Sync for UartLoggerWithBusyFlag {}

    static UART_LOGGER_WITH_BUSY_FLAG: UartLoggerWithBusyFlag = UartLoggerWithBusyFlag {
        busy: AtomicBool::new(false),
        uart: UnsafeCell::new(None),
    };

    struct UartGuard<'lock>(&'lock AtomicBool);

    impl<'lock> UartGuard<'lock> {
        pub fn new(flag: &'lock AtomicBool) -> Option<Self> {
            // This can happen if we log inside an ISR and the thread code was busy logging.
            // Simply skip logging here for thread-safety.
            if flag.swap(true, core::sync::atomic::Ordering::AcqRel) {
                return None;
            }
            Some(Self(flag))
        }
    }

    impl Drop for UartGuard<'_> {
        fn drop(&mut self) {
            self.0.store(false, core::sync::atomic::Ordering::Release);
        }
    }

    /// Initialize the logger with a blocking UART instance which spins on a busy flag in threaded
    /// mode, and does not log in interrupt contexts if the main task was busy with logging.
    ///
    /// It should be noted that this is still a blocking logger, and using it in an ISR might
    /// invalidate application logic and introduce problematic delays in the system.
    ///
    /// Therefore, the initialization also allows skipping logging in ISRs completely.
    pub fn init_with_busy_flag(uart: TxWithShiftRegister, level: LevelFilter) {
        if LOGGER_INIT_DONE.swap(true, core::sync::atomic::Ordering::Relaxed) {
            return;
        }
        LOG_SEL.swap(
            LOG_SEL_UNSAFE_SINGLE_CORE,
            core::sync::atomic::Ordering::Relaxed,
        );
        let opt_uart = unsafe { &mut *UART_LOGGER_WITH_BUSY_FLAG.uart.get() };
        opt_uart.replace(uart);

        set_logger(&UART_LOGGER_WITH_BUSY_FLAG).unwrap();
        set_max_level(level); // Adjust as needed
    }

    impl log::Log for UartLoggerWithBusyFlag {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            let guard = UartGuard::new(&self.busy);
            if guard.is_none() {
                return;
            }

            let uart_mut = unsafe { &mut *self.uart.get() }.as_mut();
            if uart_mut.is_none() {
                return;
            }

            writeln!(
                uart_mut.unwrap(),
                "{} - {}\r",
                record.level(),
                record.args()
            )
            .unwrap();
        }

        fn flush(&self) {
            let guard = UartGuard::new(&self.busy);
            if guard.is_none() {
                return;
            }

            let uart_mut = unsafe { &mut *self.uart.get() }.as_mut();
            if uart_mut.is_none() {
                return;
            }
            uart_mut.unwrap().flush().unwrap();
        }
    }

    // Flush the selected logger instance.
    pub fn flush() {
        match LOG_SEL.load(core::sync::atomic::Ordering::Relaxed) {
            val if val == LOG_SEL_LOCKED => UART_LOGGER_BLOCKING.flush(),
            val if val == LOG_SEL_UNSAFE_SINGLE_CORE => UART_LOGGER_WITH_BUSY_FLAG.flush(),
            _ => (),
        }
    }
}
