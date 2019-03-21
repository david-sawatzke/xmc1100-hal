#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use xmc1100_hal as hal;

use crate::hal::prelude::*;
use crate::hal::scu::Scu;
use crate::hal::time::{Hertz, MegaHertz};
use crate::hal::timers::*;
use crate::hal::xmc1100;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(_cp)) = (xmc1100::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let port1 = p.PORT1.split();

            let scu = Scu::new(p.SCU_GENERAL, p.SCU_CLK).freeze();
            /* (Re-)configure PA1 as output */
            let mut led = port1.p1_1.into_push_pull_output(&cs);

            /* Get timer */
            let mut timer = Timer::timer(p.CCU40_CC40, Hertz(1), &scu);
            loop {
                timer.start(Hertz(1));
                led.set_high();
                nb::block!(timer.wait()).unwrap();
                led.set_low();
                nb::block!(timer.wait()).unwrap();
                // Do "pwm"
                timer.start(Hertz(600));
                for _ in 0..200 {
                    led.set_high();
                    nb::block!(timer.wait()).unwrap();
                    nb::block!(timer.wait()).unwrap();
                    led.set_low();
                    nb::block!(timer.wait()).unwrap();
                }
            }
        });
    }

    loop {
        continue;
    }
}
