//! API for the integrated USART ports
//!
//! This only implements the usual asynchronous bidirectional 8-bit transfers.
//!
//! It's possible to use a read-only/write-only serial implementation with
//! `usartXrx`/`usartXtx`.
//!
//! # Examples
//! Echo
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::prelude::*;
//! use crate::hal::serial::Serial;
//! use crate::hal::stm32;
//!
//! use nb::block;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let rcc = p.RCC.configure().sysclk(48.mhz()).freeze();
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let tx = gpioa.pa9.into_alternate_af1(cs);
//!     let rx = gpioa.pa10.into_alternate_af1(cs);
//!
//!     let mut serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc);
//!
//!     loop {
//!         let received = block!(serial.read()).unwrap();
//!         block!(serial.write(received)).ok();
//!     }
//! });
//! ```
//!
//! Hello World
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::prelude::*;
//! use crate::hal::serial::Serial;
//! use crate::hal::stm32;
//!
//! use nb::block;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let rcc = p.RCC.configure().sysclk(48.mhz()).freeze();
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let tx = gpioa.pa9.into_alternate_af1(cs);
//!
//!     let mut serial = Serial::usart1tx(p.USART1, tx, 115_200.bps(), &mut rcc);
//!
//!     loop {
//!         serial.write_str("Hello World!\r\n");
//!     }
//! });
//! ```

use core::{
    fmt::{Result, Write},
    ops::Deref,
    ptr,
};

use embedded_hal::prelude::*;

use crate::{gpio::*, rcc::Rcc, time::Bps};

use core::marker::PhantomData;

use xmc1100;

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

pub trait TxPin<USART> {}
pub trait RxPin<USART> {}

/// Serial abstraction
pub struct Serial<USART, TXPIN, RXPIN> {
    usart: USART,
    pins: (TXPIN, RXPIN),
}

// Common register
type SerialRegisterBlock = crate::xmc1100::usic0_ch0::RegisterBlock;

/// Serial receiver
pub struct Rx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Rx<USART> {}

/// Serial transmitter
pub struct Tx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Tx<USART> {}

macro_rules! usart {
    ($($USART:ident: ($usart:ident, $usarttx:ident, $usartrx:ident, $usartXen:ident, $apbenr:ident),)+) => {
        $(
            use crate::xmc1100::$USART;
            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN>
            // where
            //     TXPIN: TxPin<$USART>,
            //     RXPIN: RxPin<$USART>,
            {
                /// Creates a new serial instance
                pub fn $usart(usart: $USART, pins: (TXPIN, RXPIN), baud_rate: Bps) -> Self {
                    let mut serial = Serial { usart, pins };
                    serial.configure(baud_rate);
                    // TODO Enable transmission and receiving
                    serial
                }
            }

            impl<TXPIN> Serial<$USART, TXPIN, ()>
            where
                TXPIN: TxPin<$USART>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usarttx(usart: $USART, txpin: TXPIN, baud_rate: Bps) -> Self {
                    let rxpin = ();
                    let mut serial = Serial {
                        usart,
                        pins: (txpin, rxpin),
                    };
                    serial.configure(baud_rate);
                    // TODO Enable transmission
                    serial
                }
            }

            impl<RXPIN> Serial<$USART, (), RXPIN>
            where
                RXPIN: RxPin<$USART>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usartrx(usart: $USART, rxpin: RXPIN, baud_rate: Bps) -> Self {
                    let txpin = ();
                    let mut serial = Serial {
                        usart,
                        pins: (txpin, rxpin),
                    };
                    serial.configure(baud_rate);
                    // TODO Enable receiving
                    serial
                }
            }

            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN> {
                fn configure(&mut self, _baud_rate: Bps) {
                    use core::mem::transmute;
                    use xmc1100::usic0_ch0::{PCR, PCR_ASCMODE};
                    // TODO Maybe enable clock & reset?
                    // XMC 1100 with 115200 8 n
                    // Enable module
                    self.usart
                        .kscfg
                        .write(|w| w.moden().set_bit().bpmoden().set_bit());
                    // Force a read, recommended by the datasheet
                    self.usart.kscfg.read();
                    // Configure the fractional divider
                    // fFD = fPB
                    unsafe { self.usart.fdr.write(|w| w.dm().value3().step().bits(590)) };
                    // Use the fractional divider as the clock source and an additional divider
                    // Configure baud rate generator
                    // BAUDRATE = fCTQIN/(BRG.PCTQ x BRG.DCTQ)
                    // CLKSEL = 0 (fPIN = fFD), CTQSEL = 00b (fCTQIN = fPDIV), PPPEN = 0
                    // (fPPP=fPIN)
                    unsafe {
                        self.usart.brg.write(|w| {
                            w.clksel()
                                .value1()
                                .pctq()
                                .bits(0)
                                .dctq()
                                .bits(9)
                                .pdiv()
                                .bits(3)
                        })
                    };
                    // USIC Shift Control
                    // SCTR.FLE = 8 (Frame Length)
                    // SCTR.WLE = 8 (Word Length)
                    // SCTR.TRM = 1 (Transmission Mode)
                    // SCTR.PDL = 1 (This bit defines the output level at the shift data output
                    // signal when no data is available for transmission)
                    unsafe {
                        self.usart
                            .sctr
                            .write(|w| w.pdl().set_bit().trm().value2().fle().bits(7).wle().bits(7))
                    };
                    // Configuration of USIC Transmit Control/Status Register
                    // TBUF.TDEN = 1 (TBUF Data Enable: A transmission of the data word in TBUF
                    //  can be started if TDV = 1
                    // TBUF.TDSSM = 1 (Data Single Shot Mode: allow word-by-word data transmission
                    //  which avoid sending the same data several times
                    self.usart
                        .tcsr
                        .write(|w| w.tdssm().set_bit().tden().bits(1));
                    // Configuration of Protocol Control Register
                    // PCR.SMD = 1 (Sample Mode based on majority)
                    // PCR.STPB = 0 (1x Stop bit)
                    // PCR.SP = 5 (Sample Point)
                    // PCR.PL = 0 (Pulse Length is equal to the bit length)

                    unsafe {
                        (*transmute::<*const PCR, *const PCR_ASCMODE>(&self.usart.pcr))
                            .write(|w| w.smd().set_bit().sp().bits(9))
                    };
                    // Configure Transmit Buffer
                    // Standard transmit buffer event is enabled
                    // Define start entry of Transmit Data FIFO buffer DPTR = 0
                    // Set Transmit Data Buffer to 32 and set data pointer to position 0
                    // Set usic ASC mode
                    unsafe { self.usart.tbctr.write(|w| w.size().value6().dptr().bits(0)) };
                    // TODO P2.2 as input (UART_RX DX0)
                    // Select P2.2 as input for USIC DX3 -> UCIC DX0
                    self.usart.dx3cr.write(|w| w.dsel().value1());
                    // Route USIC DX3 input signal to USIC DX0 (DSEL=DX0G)
                    self.usart.dx0cr.write(|w| w.dsel().value7());
                    // Configure Receive Buffer
                    // Standard Receive buffer event is enabled
                    // Define start entry of Receive Data FIFO buffer DPTR = 32
                    // Set Receive Data Buffer Size to 32 and set data pointer to position 32
                    unsafe { self.usart
                        .rbctr
                        .write(|w| w.size().value6().dptr().bits(32)) };
                    // TODO UART_TX AF6 P2.1 as output controlled by ALT6 = U0C0.DOUT0
                    // Configuration of Channel Control Register
                    // CCR.PM = 00 ( Disable parity generation)
                    // CCR.MODE = 2 (ASC mode enabled. Note: 0 (USIC channel is disabled))
                    self.usart.ccr.write(|w| w.mode().value3().pm().value1());
                    // TODO digital mode for pins
                }
            }
        )+
    }
}

usart! {
    USIC0_CH0: (usic0_ch0, usart2tx, usart2rx,usart2en, apb1enr),
    USIC0_CH1: (usic0_ch1, usart1tx, usart1rx, usart1en, apb2enr),
}

impl<USART> embedded_hal::serial::Read<u8> for Rx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(self.usart)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Read<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    RXPIN: RxPin<USART>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(&*self.usart)
    }
}

impl<USART> embedded_hal::serial::Write<u8> for Tx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = void::Void;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Write<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    // TXPIN: TxPin<USART>,
{
    type Error = void::Void;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(&*self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(&*self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    /// Splits the UART Peripheral in a Tx and an Rx part
    /// This is required for sending/receiving
    pub fn split(self) -> (Tx<USART>, Rx<USART>)
    where
        TXPIN: TxPin<USART>,
        RXPIN: RxPin<USART>,
    {
        (
            Tx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
            Rx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
        )
    }

    pub fn release(self) -> (USART, (TXPIN, RXPIN)) {
        (self.usart, self.pins)
    }
}

impl<USART> Write for Tx<USART>
where
    Tx<USART>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

impl<USART, TXPIN, RXPIN> Write for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    // TXPIN: TxPin<USART>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

/// Ensures that none of the previously written words are still buffered
fn flush(usart: *const SerialRegisterBlock) -> nb::Result<(), void::Void> {
    // TODO
    // // NOTE(unsafe) atomic read with no side effects
    // let isr = unsafe { (*usart).isr.read() };

    // if isr.tc().bit_is_set() {
    //     Ok(())
    // } else {
    //     Err(nb::Error::WouldBlock)
    // }
    Err(nb::Error::WouldBlock)
}

/// Tries to write a byte to the UART
/// Fails if the transmit buffer is full
fn write(usart: *const SerialRegisterBlock, byte: u8) -> nb::Result<(), void::Void> {
    // NOTE(unsafe) atomic read with no side effects
    let trbsr = unsafe { (*usart).trbsr.read() };

    if trbsr.tfull().bit_is_clear() {
        // Write into first fifo buffer
        unsafe { (*usart).in_[0].write(|w| w.tdata().bits(byte as u16)) };
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to read a byte from the UART
fn read(usart: *const SerialRegisterBlock) -> nb::Result<u8, Error> {
    // TODO Error out for now
    Err(nb::Error::WouldBlock)
    // // NOTE(unsafe) atomic read with no side effects
    // let isr = unsafe { (*usart).isr.read() };

    // Err(if isr.pe().bit_is_set() {
    //     nb::Error::Other(Error::Parity)
    // } else if isr.fe().bit_is_set() {
    //     nb::Error::Other(Error::Framing)
    // } else if isr.nf().bit_is_set() {
    //     nb::Error::Other(Error::Noise)
    // } else if isr.ore().bit_is_set() {
    //     nb::Error::Other(Error::Overrun)
    // } else if isr.rxne().bit_is_set() {
    //     // NOTE(read_volatile) see `write_volatile` below
    //     return Ok(unsafe { ptr::read_volatile(&(*usart).rdr as *const _ as *const _) });
    // } else {
    //     nb::Error::WouldBlock
    // })
}
