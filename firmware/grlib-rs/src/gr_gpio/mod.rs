pub mod regs;

pub use embedded_hal::digital::PinState;

pub struct Pin<const OFFSET: usize> {}

pub struct Pins {
    pub p0: Pin<0>,
    pub p1: Pin<1>,
    pub p2: Pin<2>,
    pub p3: Pin<3>,
    pub p4: Pin<4>,
    pub p5: Pin<5>,
    pub p6: Pin<6>,
    pub p7: Pin<7>,
    pub p8: Pin<8>,
    pub p9: Pin<9>,
    pub p10: Pin<10>,
    pub p11: Pin<11>,
    pub p12: Pin<12>,
    pub p13: Pin<13>,
    pub p14: Pin<14>,
    pub p15: Pin<15>,
    pub p16: Pin<16>,
    pub p17: Pin<17>,
    pub p18: Pin<18>,
    pub p19: Pin<19>,
}

impl Pins {
    fn new(_regs: &regs::MmioRegisters<'static>) -> Self {
        Self {
            p0: Pin {},
            p1: Pin {},
            p2: Pin {},
            p3: Pin {},
            p4: Pin {},
            p5: Pin {},
            p6: Pin {},
            p7: Pin {},
            p8: Pin {},
            p9: Pin {},
            p10: Pin {},
            p11: Pin {},
            p12: Pin {},
            p13: Pin {},
            p14: Pin {},
            p15: Pin {},
            p16: Pin {},
            p17: Pin {},
            p18: Pin {},
            p19: Pin {},
        }
    }
}

pub struct Gpio {
    regs: regs::MmioRegisters<'static>,
}

impl Gpio {
    pub fn new(regs: regs::MmioRegisters<'static>) -> (Self, Pins) {
        let pins = Pins::new(&regs);
        (Self { regs }, pins)
    }

    pub fn input_pin<const OFFSET: usize>(&self, pin: Pin<OFFSET>) -> Input {
        Input::new(unsafe { self.regs.clone() }, pin)
    }

    pub fn output_pin<const OFFSET: usize>(
        &self,
        pin: Pin<OFFSET>,
        init_level: embedded_hal::digital::PinState,
    ) -> Output {
        Output::new(unsafe { self.regs.clone() }, pin, init_level)
    }
}

pub struct LowLevelPin {
    offset: u8,
    regs: regs::MmioRegisters<'static>,
}

impl LowLevelPin {
    pub fn new<const OFFSET: usize>(
        regs: &regs::MmioRegisters<'static>,
        _pin: Pin<OFFSET>,
    ) -> Self {
        Self::steal(regs, OFFSET)
    }

    pub fn steal(regs: &regs::MmioRegisters<'static>, offset: usize) -> Self {
        Self {
            offset: offset as u8,
            regs: unsafe { regs.clone() },
        }
    }

    pub fn set_input_enable(&mut self, enable: bool) {
        if enable {
            self.regs
                .modify_input_enable(|val| val | (1 << self.offset));
        } else {
            self.regs
                .modify_input_enable(|val| val & !(1 << self.offset));
        }
    }

    #[inline(always)]
    pub fn enable_output(&mut self) {
        self.regs.modify_dir(|val| val | (1 << self.offset));
    }

    #[inline(always)]
    pub fn disable_output(&mut self) {
        self.regs.modify_dir(|val| val & !(1 << self.offset));
    }

    #[inline(always)]
    pub fn is_set_high(&self) -> bool {
        (self.regs.read_output() >> self.offset) & 1 != 0
    }

    #[inline(always)]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    #[inline(always)]
    pub fn is_high(&self) -> bool {
        (self.regs.read_data() >> self.offset) & 1 != 0
    }

    #[inline(always)]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    #[inline(always)]
    pub fn set_high(&mut self) {
        self.regs.write_output_logic_or((1 << self.offset) as u32);
        //self.regs.modify_output(|val| val | (1 << self.offset));
    }

    #[inline(always)]
    pub fn set_low(&mut self) {
        self.regs.write_output_logic_and(!(1 << self.offset) as u32);
        //self.regs.modify_output(|val| val & !(1 << self.offset));
    }

    #[inline(always)]
    pub fn toggle(&mut self) {
        self.regs.write_output_logic_xor((1 << self.offset) as u32);
    }
}

pub struct Input(LowLevelPin);

impl Input {
    /// Constructor for an input pin.
    ///
    /// It is recommended to use [Gpio::input_pin] to retrieve an instance of an input pin.
    pub fn new<const OFFSET: usize>(regs: regs::MmioRegisters<'static>, pin: Pin<OFFSET>) -> Self {
        let mut pin = LowLevelPin::new(&regs, pin);
        pin.set_input_enable(true);
        pin.disable_output();
        Self(pin)
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        self.0.is_high()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.0.is_low()
    }
}

impl embedded_hal::digital::ErrorType for Input {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::InputPin for Input {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.0.is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.0.is_low())
    }
}

pub struct Output(LowLevelPin);

impl Output {
    /// Constructor for an input pin.
    ///
    /// It is recommended to use [Gpio::output_pin] to retrieve an instance of an input pin.
    pub fn new<const OFFSET: usize>(
        regs: regs::MmioRegisters<'static>,
        pin: Pin<OFFSET>,
        init_level: PinState,
    ) -> Self {
        let mut pin = LowLevelPin::new(&regs, pin);
        match init_level {
            PinState::Low => pin.set_low(),
            PinState::High => pin.set_high(),
        }
        pin.enable_output();
        Self(pin)
    }

    #[inline]
    pub fn set_high(&mut self) {
        self.0.set_high();
    }

    #[inline]
    pub fn set_low(&mut self) {
        self.0.set_low();
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.0.toggle();
    }
}

impl embedded_hal::digital::ErrorType for Output {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for Output {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl embedded_hal::digital::StatefulOutputPin for Output {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.0.is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.0.is_set_low())
    }
}
