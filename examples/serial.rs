#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use xmc1100_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::scu::Scu;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use crate::hal::xmc1100;
use core::fmt::Write;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (xmc1100::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let port1 = p.PORT1.split();
            let port2 = p.PORT2.split();

            let mut scu = Scu::new(p.SCU_GENERAL, p.SCU_CLK);

            // (Re-)configure PA1 as output
            let mut led = port1.p1_1.into_push_pull_output(&cs);
            let rx = port2.p2_2.into_floating_input(&cs);
            let tx = port2.p2_0.into_alternate_af6(&cs);
            let tx = port2.p2_1.into_alternate_af6(&cs);

            // Get delay provider
            let mut delay = Delay::new(cp.SYST, &scu);
            // Create usart
            let mut serial = Serial::usic0_ch0(p.USIC0_CH0, (tx, ()), Bps(9600), &mut scu);
            loop {
                led.set_high();
                serial.write_str("Off\r\n");
                delay.delay_ms(1_000_u16);
                led.set_low();
                serial.write_str("On\r\n");
                delay.delay_ms(1_000_u16);
            }
        });
    }

    loop {
        continue;
    }
}
