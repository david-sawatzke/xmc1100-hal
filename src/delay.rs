//! API for delays with the systick timer
//!
//! Please be aware of potential overflows.
//! For example, the maximum delay with 48MHz is around 89 seconds
//!
//! Consider using the timers api as a more flexible interface
//!
//! # Example
//!
//! TODO Look in the `examples/` directory

use cast::{u16, u32};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use crate::scu::Scu;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

/// System timer (SysTick) as a delay provider
#[derive(Clone)]
pub struct Delay {
    scale: u32,
}

const SYSTICK_RANGE: u32 = 0x0100_0000;

impl Delay {
    /// Configures the system timer (SysTick) as a delay provider
    /// As access to the count register is possible without a reference, we can
    /// just drop it
    pub fn new(mut syst: SYST, scu: &Scu) -> Delay {
        syst.set_clock_source(SystClkSource::Core);

        syst.set_reload(SYSTICK_RANGE - 1);
        syst.clear_current();
        syst.enable_counter();
        // TODO Check on which clock we're running
        assert!(scu.clocks.sysclk().0 >= 1_000_000);
        let scale = scu.clocks.sysclk().0 / 1_000_000;

        Delay { scale }
    }
}

impl DelayMs<u32> for Delay {
    // At 48 MHz, calling delay_us with ms * 1_000 directly overflows at 0x15D868 (just over the max u16 value)
    fn delay_ms(&mut self, mut ms: u32) {
        const MAX_MS: u32 = 0x0000_FFFF;
        while ms != 0 {
            let current_ms = if ms <= MAX_MS { ms } else { MAX_MS };
            self.delay_us(current_ms * 1_000);
            ms -= current_ms;
        }
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_us(u32::from(ms) * 1_000);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u16(ms));
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        // The SysTick Reload Value register supports values between 1 and 0x00FFFFFF.
        // Here less than maximum is used so we have some play if there's a long running interrupt.
        const MAX_TICKS: u32 = 0x007F_FFFF;

        let mut total_ticks = us * self.scale;

        while total_ticks != 0 {
            let current_ticks = if total_ticks <= MAX_TICKS {
                total_ticks
            } else {
                MAX_TICKS
            };

            let start_count = SYST::get_current();
            total_ticks -= current_ticks;

            // Use the wrapping substraction and the modulo to deal with the systick wrapping around
            // from 0 to 0xFFFF
            while (start_count.wrapping_sub(SYST::get_current()) % SYSTICK_RANGE) < current_ticks {}
        }
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(u32(us))
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(u32(us))
    }
}
