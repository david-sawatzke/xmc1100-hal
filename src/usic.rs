use crate::{scu::Scu, time::Bps};
use core::marker::PhantomData;
use core::ops::Deref;
use xmc1100;

pub trait Dout0Pin<USIC> {}

pub trait Dx0Pin<USIC> {
    fn number() -> u8;
}
pub trait Dx1Pin<USIC> {
    fn number() -> u8;
}
pub trait Dx2Pin<USIC> {
    fn number() -> u8;
}
pub trait Dx3Pin<USIC> {
    fn number() -> u8;
}
pub trait Dx4Pin<USIC> {
    fn number() -> u8;
}
pub trait Dx5Pin<USIC> {
    fn number() -> u8;
}

pub struct Dx0Dx3Pin<PIN, USIC> {
    pin: PIN,
    phantom: PhantomData<USIC>,
}

impl<USIC, PIN> Dx0Pin<USIC> for Dx0Dx3Pin<PIN, USIC> {
    fn number() -> u8 {
        6
    }
}
// Common register
pub(crate) type UsicRegisterBlock = xmc1100::usic0_ch0::RegisterBlock;

pub fn dx3pin_to_dx0pin<USIC, PIN>(pin: PIN, usic: &mut USIC) -> Dx0Dx3Pin<PIN, USIC>
where
    USIC: Deref<Target = UsicRegisterBlock>,
    PIN: Dx3Pin<USIC>,
{
    usic.dx3cr.write(|w| w.dsel().bits(PIN::number()));
    Dx0Dx3Pin {
        pin: pin,
        phantom: PhantomData,
    }
}

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
