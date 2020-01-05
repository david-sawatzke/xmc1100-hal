//! API for the integrated USIC ports
//!
//! This only implements the usual asynchronous bidirectional 8-bit transfers.
//!
//! It's possible to use a read-only/write-only serial implementation with
//! `usicXrx`/`usicXtx`.

use core::{
    fmt::{Result, Write},
    ops::Deref,
    ptr,
};

use embedded_hal::prelude::*;

use crate::usic::Dout0Pin;
use crate::{scu::Scu, time::Bps, usic::*};

use core::marker::PhantomData;

/// Serial error
#[derive(Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    #[doc(hidden)]
    _Extensible,
}

/// Serial abstraction
pub struct Serial<USIC, TXPIN, RXPIN> {
    usic: USIC,
    pins: (TXPIN, RXPIN),
}

/// Serial receiver
pub struct Rx<USIC> {
    usic: *const UsicRegisterBlock,
    _instance: PhantomData<USIC>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USIC> Send for Rx<USIC> {}

/// Serial transmitter
pub struct Tx<USIC> {
    usic: *const UsicRegisterBlock,
    _instance: PhantomData<USIC>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USIC> Send for Tx<USIC> {}

macro_rules! serial {
    ($($USIC:ident: ($usic:ident, $usictx:ident, $usicrx:ident),)+) => {
        $(
            use crate::xmc1100::$USIC;
            impl<TXPIN, RXPIN> Serial<$USIC, TXPIN, RXPIN>
            where
                TXPIN: Dout0Pin<$USIC>,
                RXPIN: Dx0Pin<$USIC>,
            {
                /// Creates a new serial instance
                pub fn $usic(usic: $USIC, pins: (TXPIN, RXPIN), baud_rate: Bps, scu: &mut Scu) -> Self {
                    let pin_num = RXPIN::number();
                    let mut serial = Serial { usic, pins };
                    serial.configure(baud_rate, scu);
                    // Set rx pin
                    serial.usic.dx0cr.write(|w| w.dsel().bits(pin_num));
                    // TODO Enable transmission and receiving
                    serial
                }
            }

            impl<TXPIN> Serial<$USIC, TXPIN, ()>
            where
                TXPIN: Dout0Pin<$USIC>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usictx(usic: $USIC, txpin: TXPIN, baud_rate: Bps, scu: &mut Scu) -> Self {
                    let rxpin = ();
                    let mut serial = Serial {
                        usic,
                        pins: (txpin, rxpin),
                    };
                    serial.configure(baud_rate, scu);
                    // TODO Enable transmission
                    serial
                }
            }

            impl<RXPIN> Serial<$USIC, (), RXPIN>
            where
                RXPIN: Dx0Pin<$USIC>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usicrx(usic: $USIC, rxpin: RXPIN, baud_rate: Bps, scu: &mut Scu) -> Self {
                    let txpin = ();
                    let pin_num = RXPIN::number();
                    // Set rx pin
                    let mut serial = Serial {
                        usic,
                        pins: (txpin, rxpin),
                    };
                    serial.configure(baud_rate, scu);
                    // Set rx pin
                    serial.usic.dx0cr.write(|w| w.dsel().bits(pin_num));
                    // TODO Enable receiving
                    serial
                }
            }

            impl<TXPIN, RXPIN> Serial<$USIC, TXPIN, RXPIN> {
                fn configure(&mut self, baud: Bps, scu: &mut Scu) {
                    // Disable clock gating
                    scu.scu_clk.cgatclr0.write(|w| w.usic0().set_bit());
                    // XMC 1100 with 115200 8 n
                    // Enable module
                    self.usic
                        .kscfg
                        .write(|w| w.moden().set_bit().bpmoden().set_bit());
                    // Force a read, recommended by the datasheet
                    self.usic.kscfg.read();

                    // Set the timing with oversampling of 16
                    crate::usic::set_baudrate(&mut self.usic, scu, baud, 16).unwrap();
                    // USIC Shift Control
                    // SCTR.FLE = 8 (Frame Length)
                    // SCTR.WLE = 8 (Word Length)
                    // SCTR.TRM = 1 (Transmission Mode)
                    // SCTR.PDL = 1 (This bit defines the output level at the shift data output
                    // signal when no data is available for transmission)
                    unsafe {
                        self.usic
                            .sctr
                            .write(|w| w.pdl().set_bit().trm().value2().fle().bits(7).wle().bits(7))
                    };
                    // Configuration of USIC Transmit Control/Status Register
                    // TBUF.TDEN = 1 (TBUF Data Enable: A transmission of the data word in TBUF
                    //  can be started if TDV = 1
                    // TBUF.TDSSM = 1 (Data Single Shot Mode: allow word-by-word data transmission
                    //  which avoid sending the same data several times
                    self.usic
                        .tcsr
                        .write(|w| w.tdssm().set_bit().tden().bits(1));
                    // Configuration of Protocol Control Register
                    // PCR.SMD = 1 (Sample Mode based on majority)
                    // PCR.STPB = 0 (1x Stop bit)
                    // PCR.SP = 5 (Sample Point)
                    // PCR.PL = 0 (Pulse Length is equal to the bit length)
                    unsafe {self.usic.pcr_ascmode_mut()
                            .write(|w| w.smd().set_bit().sp().bits(9))
                    };
                    // Configure Transmit Buffer
                    // Standard transmit buffer event is enabled
                    // Define start entry of Transmit Data FIFO buffer DPTR = 0
                    // Set Transmit Data Buffer to 32 and set data pointer to position 0
                    // Set usic ASC mode
                    unsafe { self.usic.tbctr.write(|w| w.size().value6().dptr().bits(0)) };
                    // Configure Receive Buffer
                    // Standard Receive buffer event is enabled
                    // Define start entry of Receive Data FIFO buffer DPTR = 32
                    // Set Receive Data Buffer Size to 32 and set data pointer to position 32
                    unsafe { self.usic
                        .rbctr
                        .write(|w| w.size().value6().dptr().bits(32)) };
                    // Configuration of Channel Control Register
                    // CCR.PM = 00 ( Disable parity generation)
                    // CCR.MODE = 2 (ASC mode enabled. Note: 0 (USIC channel is disabled))
                    self.usic.ccr.write(|w| w.mode().value3().pm().value1());
                }
            }
        )+
    }
}

serial! {
    USIC0_CH0: (usic0_ch0, usic0_ch0tx, usic0_ch1rx),
    USIC0_CH1: (usic0_ch1, usic0_ch1tx, usic0_ch0rx),
}

impl<USIC> embedded_hal::serial::Read<u8> for Rx<USIC>
where
    USIC: Deref<Target = UsicRegisterBlock>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(self.usic)
    }
}

impl<USIC, TXPIN, RXPIN> embedded_hal::serial::Read<u8> for Serial<USIC, TXPIN, RXPIN>
where
    USIC: Deref<Target = UsicRegisterBlock>,
    RXPIN: Dx0Pin<USIC>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(&*self.usic)
    }
}

impl<USIC> embedded_hal::serial::Write<u8> for Tx<USIC>
where
    USIC: Deref<Target = UsicRegisterBlock>,
{
    type Error = void::Void;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(self.usic)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(self.usic, byte)
    }
}

impl<USIC, TXPIN, RXPIN> embedded_hal::serial::Write<u8> for Serial<USIC, TXPIN, RXPIN>
where
    USIC: Deref<Target = UsicRegisterBlock>,
    TXPIN: Dout0Pin<USIC>,
{
    type Error = void::Void;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(&*self.usic)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(&*self.usic, byte)
    }
}

impl<USIC, TXPIN, RXPIN> Serial<USIC, TXPIN, RXPIN>
where
    USIC: Deref<Target = UsicRegisterBlock>,
{
    /// Splits the UART Peripheral in a Tx and an Rx part
    /// This is required for sending/receiving
    pub fn split(self) -> (Tx<USIC>, Rx<USIC>)
    where
        TXPIN: Dout0Pin<USIC>,
        USIC: Deref<Target = UsicRegisterBlock>,
    {
        (
            Tx {
                usic: &*self.usic,
                _instance: PhantomData,
            },
            Rx {
                usic: &*self.usic,
                _instance: PhantomData,
            },
        )
    }

    pub fn release(self) -> (USIC, (TXPIN, RXPIN)) {
        (self.usic, self.pins)
    }
}

impl<USIC> Write for Tx<USIC>
where
    Tx<USIC>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

impl<USIC, TXPIN, RXPIN> Write for Serial<USIC, TXPIN, RXPIN>
where
    USIC: Deref<Target = UsicRegisterBlock>,
    TXPIN: Dout0Pin<USIC>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

/// Ensures that none of the previously written words are still buffered
fn flush(usic: *const UsicRegisterBlock) -> nb::Result<(), void::Void> {
    // NOTE(unsafe) atomic read with no side effects
    let psr = unsafe { &(*usic).psr_ascmode().read() };
    if psr.txidle().bit_is_set() {
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to write a byte to the UART
/// Fails if the transmit buffer is full
fn write(usic: *const UsicRegisterBlock, byte: u8) -> nb::Result<(), void::Void> {
    // NOTE(unsafe) atomic read with no side effects
    let trbsr = unsafe { (*usic).trbsr.read() };

    if trbsr.tfull().bit_is_clear() {
        // Write into first fifo buffer
        unsafe { (*usic).in_[0].write(|w| w.tdata().bits(byte as u16)) };
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to read a byte from the UART
fn read(usic: *const UsicRegisterBlock) -> nb::Result<u8, Error> {
    // NOTE(unsafe) atomic read with no side effects
    let trbsr = unsafe { (*usic).trbsr.read() };
    // NOTE(unsafe) atomic read with no side effects
    let psr = unsafe { &(*usic).psr_ascmode().read() };
    Err(
        // TODO Detect Parity error
        if psr.fer0().bit_is_set() || psr.fer1().bit_is_set() {
            nb::Error::Other(Error::Framing)
        } else if psr.rns().bit_is_set() {
            nb::Error::Other(Error::Noise)
        } else if trbsr.rfull().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if trbsr.rempty().bit_is_clear() {
            // NOTE(read_volatile) see `write_volatile` below
            return Ok(unsafe { ptr::read_volatile(&(*usic).outr as *const _ as *const _) });
        } else {
            nb::Error::WouldBlock
        },
    )
}
