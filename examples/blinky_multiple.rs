#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use xmc1100_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::scu::Scu;
use crate::hal::xmc1100;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (xmc1100::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let port1 = p.PORT1.split();

            let scu = Scu::new(p.SCU_GENERAL, p.SCU_CLK).freeze();
            /* (Re-)configure P1.0 as output */
            let mut led1 = port1.p1_0.into_push_pull_output(&cs);
            /* (Re-)configure P1.1 as output */
            let led2 = port1.p1_1.into_push_pull_output(&cs);
            led1.set_high().ok();
            let mut leds = [led1.downgrade(), led2.downgrade()];

            /* Get delay provider */
            let mut delay = Delay::new(cp.SYST, &scu);
            loop {
                for led in leds.iter_mut() {
                    led.toggle().ok();
                }
                delay.delay_ms(1_000_u16);
            }
        });
    }

    loop {
        continue;
    }
}
