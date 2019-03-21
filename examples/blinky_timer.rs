#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use xmc1100_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::scu::Scu;
use crate::hal::time::Hertz;
use crate::hal::timers::*;
use crate::hal::xmc1100;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (xmc1100::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let port1 = p.PORT1.split();

            let scu = Scu::new(p.SCU_GENERAL, p.SCU_CLK);
            /* (Re-)configure PA1 as output */
            let mut led = port1.p1_1.into_push_pull_output(&cs);

            /* Get timer */
            let mut timer = Timer::timer(p.CCU40_CC40, Hertz(1), &scu);
            loop {
                timer.start(Hertz(1));
                led.set_high();
                nb::block!(timer.wait());
                led.set_low();
                nb::block!(timer.wait());
                timer.start(Hertz(20));
                led.set_high();
                nb::block!(timer.wait());
                led.set_low();
                nb::block!(timer.wait());
                timer.start(Hertz(500));
                for _ in 0..250 {
                    led.set_high();
                    nb::block!(timer.wait());
                    led.set_low();
                    nb::block!(timer.wait());
                }
            }
        });
    }

    loop {
        continue;
    }
}
