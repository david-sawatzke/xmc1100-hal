#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use xmc1100_hal as hal;

use crate::hal::prelude::*;
use crate::hal::xmc1100;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = xmc1100::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let port1 = p.PORT1.split();

            /* (Re-)configure PA1 as output */
            let mut led = port1.p1_1.into_push_pull_output(&cs);

            loop {
                /* Turn PA1 on a million times in a row */
                for _ in 0..1_000_000 {
                    led.set_high();
                }
                /* Then turn PA1 off a million times in a row */
                for _ in 0..1_000_000 {
                    led.set_low();
                }
            }
        });
    }

    loop {
        continue;
    }
}
