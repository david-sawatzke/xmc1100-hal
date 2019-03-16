use crate::{scu::Scu, time::Bps};
use core::ops::Deref;
use xmc1100;

pub trait Dout0Pin<USIC> {}
// Common register
type UsicRegisterBlock = xmc1100::usic0_ch0::RegisterBlock;

pub(crate) fn set_baudrate<USIC>(
    usic: &mut USIC,
    scu: &mut Scu,
    bps: Bps,
    oversampling: u8,
) -> Result<(), ()>
where
    USIC: Deref<Target = UsicRegisterBlock>,
{
    // Pretty much the code from XMCLib
    let peripheral_clock = scu.clocks.sysclk().0 / 100;
    let mut clock_divider_min = 1;
    let mut pdiv_int_min = 1;
    let mut pdiv_frac_min = 0x3FF;
    if bps.0 < 100 {
        return Err(());
    }
    let rate = bps.0 / 100;
    for clock_divider in (1..1024 as u32).rev() {
        let pdiv = (peripheral_clock * clock_divider) / (rate * oversampling as u32);
        let pdiv_int = pdiv >> 10;
        let pdiv_frac = pdiv & 0x3ff;
        if (pdiv_int < 1024) && (pdiv_frac < pdiv_frac_min) {
            pdiv_frac_min = pdiv_frac;
            pdiv_int_min = pdiv_int;
            clock_divider_min = clock_divider;
        }
    }
    unsafe {
        usic.fdr
            .write(|w| w.dm().value3().step().bits(clock_divider_min as u16))
    };
    unsafe {
        usic.brg.write(|w| {
            w.clksel()
                .value1()
                .pctq()
                .bits(0)
                .dctq()
                .bits(oversampling - 1)
                .pdiv()
                .bits(pdiv_int_min as u16 - 1)
        })
    };
    Ok(())
}
