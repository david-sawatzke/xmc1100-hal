#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use xmc1100_hal as hal;

use crate::hal::gpio::*;
use crate::hal::prelude::*;
use crate::hal::xmc1100;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::syst::SystClkSource::Core;
use cortex_m::peripheral::Peripherals;
use cortex_m_rt::{entry, exception};

use core::cell::RefCell;
use core::ops::DerefMut;
use xmc1100_hal::rcc::Clocks;
use xmc1100_hal::time::MegaHertz;

static PORT: Mutex<RefCell<Option<port1::P1_1<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (xmc1100::Peripherals::take(), Peripherals::take()) {
        let port1 = p.PORT1.split();

        let _clocks = Clocks {
            sysclk: MegaHertz(8).into(),
        };

        let mut syst = cp.SYST;

        cortex_m::interrupt::free(move |cs| {
            /* (Re-)configure PA1 as output */
            let led = port1.p1_1.into_push_pull_output(&cs);
            *PORT.borrow(cs).borrow_mut() = Some(led);
        });

        /* Initialise SysTick counter with a defined value */
        unsafe { syst.cvr.write(1) };

        /* Set source for SysTick counter, here full operating frequency (== 64MHz) */
        syst.set_clock_source(Core);

        /* Set reload value, i.e. timer delay 8 MHz/8 Mcounts == 1Hz or 1 s */
        syst.set_reload(256_000 - 1);

        /* Start counter */
        syst.enable_counter();

        /* Start interrupt generation */
        syst.enable_interrupt();

        loop {}
    }
    loop {
        continue;
    }
}

/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the flash function will be called and the specified stated passed in via argument */
//, flash, state: u8 = 1);
#[exception]
fn SysTick() -> ! {
    static mut state: u8 = 0;

    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = *PORT.borrow(cs).borrow_mut().deref_mut() {
            /* Check state variable, keep LED off most of the time and turn it on every 10th tick */
            if *state < 0x7F {
                /* If set turn off the LED */
                led.set_high();
            } else {
                /* If not set, turn on the LED */
                led.set_low();
            }
            *state += 1;
        }
    });
}
