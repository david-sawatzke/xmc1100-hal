#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use xmc1100_hal as hal;

use crate::hal::prelude::*;
use crate::hal::scu::Scu;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use crate::hal::usic;
use crate::hal::xmc1100;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = xmc1100::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let port2 = p.PORT2.split();
            let mut usic = p.USIC0_CH0;

            let mut scu = Scu::new(p.SCU_GENERAL, p.SCU_CLK).freeze();

            let rx = port2.p2_2.into_floating_input(&cs);
            // Used so output can be sniffed
            let _tx = port2.p2_0.into_alternate_af6(&cs);
            let tx = port2.p2_1.into_alternate_af6(&cs);
            let rx = usic::dx3pin_to_dx0pin(rx, &mut usic);
            // Create usart
            let mut serial = Serial::usic0_ch0(usic, (tx, rx), Bps(9600), &mut scu);
            loop {
                // Wait for reception of a single byte
                let received = nb::block!(serial.read()).unwrap();

                // Send back previously received byte and wait for completion
                nb::block!(serial.write(received)).ok();
            }
        });
    }

    loop {
        continue;
    }
}
