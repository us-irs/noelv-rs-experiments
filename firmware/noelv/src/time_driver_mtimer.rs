use core::cell::{Cell, RefCell};

use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::{Driver as _, time_driver_impl};
use embassy_time_queue_utils::Queue;
use once_cell::sync::OnceCell;

unsafe impl Send for AlarmState {}

// The timer runs at a fixed frequency and needs to be scaled to the tick frequency
// of embassy using a scale.
static SCALE: OnceCell<u32> = OnceCell::new();

/// Machine timer interrupt triggers when the compare value is reached.
#[riscv_rt::core_interrupt(riscv::interrupt::Interrupt::MachineTimer)]
fn machine_timer_handler() {
    unsafe {
        TIME_DRIVER.on_interrupt();
    }
}

/// This is the initialization method for the embassy time driver.
///
/// It should be called ONCE at system initialization.
pub fn init(sys_clk_hz: u32, enable_global_interrupts: bool) {
    if SCALE.get().is_some() {
        return;
    }
    unsafe { TIME_DRIVER.init(sys_clk_hz, enable_global_interrupts) };
}

#[derive(Debug)]
pub struct CoreTimerDriver {
    // Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

impl CoreTimerDriver {
    /// This is the initialization method for the embassy time driver.
    ///
    /// # Safety
    ///
    /// This has to be called ONCE at system initialization.
    pub unsafe fn init(&'static self, sys_clk_hz: u32, enable_global_interrupts: bool) {
        // The timer runs at a fixed frequency and needs to be scaled to the tick frequency
        // of embassy.
        SCALE
            .set(sys_clk_hz / 2 / embassy_time_driver::TICK_HZ as u32)
            .unwrap();
        // Enable core timer.
        crate::timer::enable_timer();
        crate::timer::write_compare_value(crate::HartId::Hart0, u64::MAX);
        unsafe {
            crate::enable_interrupts(enable_global_interrupts);
        }
    }

    /// Should be called inside the machine timer interrupt handler. This is done by the driver.
    ///
    /// # Safety
    ///
    /// This function has to be called once for interrupt ID
    /// [crate::hal::gic::PpiInterrupt::GlobalTimer].
    pub unsafe fn on_interrupt(&self) {
        critical_section::with(|cs| {
            self.trigger_alarm(cs);
        })
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        crate::timer::disable_interrupt();

        let alarm = &self.alarms.borrow(cs);
        // Setting the maximum value disables the alarm.
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        let mut next = self
            .queue
            .borrow(cs)
            .borrow_mut()
            .next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self
                .queue
                .borrow(cs)
                .borrow_mut()
                .next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        if SCALE.get().is_none() {
            return false;
        }
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        let t = self.now();
        if timestamp <= t {
            crate::timer::disable_interrupt();
            alarm.timestamp.set(u64::MAX);
            return false;
        }

        // If it hasn't triggered yet, setup the relevant reset value, regardless of whether
        // the interrupts are enabled or not. When they are enabled at a later point, the
        // right value is already set.

        // If the timestamp is in the next few ticks, add a bit of buffer to be sure the alarm
        // is not missed.
        //
        // This means that an alarm can be delayed for up to 2 ticks (from t+1 to t+3), but this is allowed
        // by the Alarm trait contract. What's not allowed is triggering alarms *before* their scheduled time,
        // and we don't do that here.
        let safe_timestamp = timestamp.max(t + 3);
        let opt_comparator = safe_timestamp.checked_mul(*SCALE.get().unwrap() as u64);
        if opt_comparator.is_none() {
            return true;
        }
        crate::timer::write_compare_value(crate::HartId::Hart0, opt_comparator.unwrap());
        crate::timer::enable_interrupt();
        true
    }
}

impl embassy_time_driver::Driver for CoreTimerDriver {
    #[inline]
    fn now(&self) -> u64 {
        // Raw tick must be scaled to embassy tick frequency.
        crate::timer::read_timer() / *SCALE.get().unwrap() as u64
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();

            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        })
    }
}
#[derive(Debug)]
struct AlarmState {
    timestamp: Cell<u64>,
}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

time_driver_impl!(
    static TIME_DRIVER: CoreTimerDriver = CoreTimerDriver {
        alarms: Mutex::new(AlarmState::new()),
        queue: Mutex::new(RefCell::new(Queue::new())),
});
