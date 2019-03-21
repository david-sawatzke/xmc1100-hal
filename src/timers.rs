//! API for the integrated timers
//!
//! This only implements basic functions, a lot of things are missing

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use crate::scu::{Clocks, Scu};
use crate::time::Hertz;
use core::ops::Deref;
use embedded_hal::timer::{CountDown, Periodic};
use nb;
use void::Void;
use xmc1100::ccu40_cc40;
use xmc1100::CCU40;

/// Hardware timers
pub struct Timer<TIM> {
    clocks: Clocks,
    tim: TIM,
}
// Systick timer
pub struct SystickTimer {
    clocks: Clocks,
    tim: SYST,
}
/// Interrupt events
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

impl SystickTimer {
    /// Configures the SYST clock as a periodic count down timer
    pub fn syst<T>(mut syst: SYST, timeout: T, scu: &Scu) -> Self
    where
        T: Into<Hertz>,
    {
        syst.set_clock_source(SystClkSource::Core);
        let mut timer = SystickTimer {
            tim: syst,
            clocks: scu.clocks,
        };
        timer.start(timeout);
        timer
    }

    /// Starts listening for an `event`
    pub fn listen(&mut self, event: &Event) {
        match event {
            Event::TimeOut => self.tim.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: &Event) {
        match event {
            Event::TimeOut => self.tim.disable_interrupt(),
        }
    }
}

/// Use the systick as a timer
///
/// Be aware that intervals less than 4 Hertz may not function properly
impl CountDown for SystickTimer {
    type Time = Hertz;

    /// Start the timer with a `timeout`
    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Hertz>,
    {
        let rvr = self.clocks.sysclk().0 / timeout.into().0 - 1;

        assert!(rvr < (1 << 24));

        self.tim.set_reload(rvr);
        self.tim.clear_current();
        self.tim.enable_counter();
    }

    /// Return `Ok` if the timer has wrapped
    /// Automatically clears the flag and restarts the time
    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.tim.has_wrapped() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Periodic for SystickTimer {}

pub(crate) type CcuRegisterBlock = ccu40_cc40::RegisterBlock;

impl<TIMER> Timer<TIMER>
where
    TIMER: Deref<Target = CcuRegisterBlock>,
{
    pub fn timer<T>(timer: TIMER, timeout: T, scu: &Scu) -> Self
    where
        T: Into<Hertz>,
    {
        // Disable clock gating
        scu.scu_clk.cgatclr0.write(|w| w.ccu40().set_bit());
        // TODO FIx this
        // Disable timer idle status
        // NOTE(unsafe) This is a write only register
        unsafe {
            (*CCU40::ptr()).gidls.write(|w| {
                w.ss0i()
                    .set_bit()
                    .ss1i()
                    .set_bit()
                    .ss2i()
                    .set_bit()
                    .ss3i()
                    .set_bit()
            })
        };
        // Enable the timers
        // NOTE(unsafe) This is a write only register
        unsafe {
            (*CCU40::ptr()).gidlc.write(|w| {
                w.cs0i()
                    .set_bit()
                    .cs1i()
                    .set_bit()
                    .cs2i()
                    .set_bit()
                    .cs3i()
                    .set_bit()
            })
        };
        // Shadow Transfer on clear
        timer.tc.write(|w| w.clst().set_bit());
        let mut timer = Timer {
            tim: timer,
            clocks: scu.clocks,
        };
        timer.start(timeout);
        // Start the timer
        timer.tim.tcset.write(|w| w.trbs().set_bit());
        // unsafe { (*CCU40::ptr()).gidls.write(|w| w.psic().set_bit()) };
        // unsafe { (*CCU40::ptr()).gidlc.write(|w| w.sprb().set_bit()) };
        timer
    }
}

impl<TIMER> CountDown for Timer<TIMER>
where
    TIMER: Deref<Target = CcuRegisterBlock>,
{
    type Time = Hertz;

    /// Start the timer with a `timeout`
    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Hertz>,
    {
        // GENERAL STRATEGY
        // Edge counting mode
        // Counts up until it hits PR, then reset to 0
        // So we need to adjust PR & (prescaling)
        // Timer period is PR + 1, so we have to substract 1
        // Use a normal prescaler (psc.psiv) 2^n
        let timeout = timeout.into();
        let ticks = self.clocks.sysclk().0 / timeout.0.max(1);
        let divider = ((ticks >> 16) + 1).next_power_of_two();
        let pr = (ticks / divider) - 1;
        unsafe { self.tim.prs.write(|w| w.prs().bits(pr as u16)) };
        // Set the prescaler
        // TODO Maybe modify next_power_of_two implementation to make this simpler
        unsafe {
            self.tim.psc.write(|w| match divider {
                1 => w.psiv().bits(0),
                2 => w.psiv().bits(1),
                4 => w.psiv().bits(2),
                8 => w.psiv().bits(3),
                16 => w.psiv().bits(4),
                32 => w.psiv().bits(5),
                64 => w.psiv().bits(6),
                128 => w.psiv().bits(7),
                256 => w.psiv().bits(8),
                512 => w.psiv().bits(9),
                1024 => w.psiv().bits(10),
                2048 => w.psiv().bits(11),
                4096 => w.psiv().bits(12),
                8192 => w.psiv().bits(13),
                16384 => w.psiv().bits(14),
                32768 => w.psiv().bits(15),
                _ => panic!("Timer prescaler value too large"),
            })
        };
        // Reset the timer count
        self.tim.tcclr.write(|w| w.tcc().set_bit());
        // Reset period match
        self.tim.swr.write(|w| w.rpm().set_bit());
        unsafe {
            (*CCU40::ptr()).gcss.write(|w| {
                w.s0se()
                    .set_bit()
                    .s1se()
                    .set_bit()
                    .s2se()
                    .set_bit()
                    .s3se()
                    .set_bit()
            })
        };
    }

    /// Return `Ok` if the timer has wrapped
    fn wait(&mut self) -> nb::Result<(), Void> {
        // Check if a period match has occured
        if self.tim.ints.read().pmus().bit_is_set() {
            // Reset period match
            self.tim.swr.write(|w| w.rpm().set_bit());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<TIMER> Periodic for Timer<TIMER> where TIMER: Deref<Target = CcuRegisterBlock> {}
