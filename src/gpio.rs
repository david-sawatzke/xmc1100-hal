//! General Purpose Input / Output

use core::marker::PhantomData;

// TODO Implement marker for af with PushPull or OpenDrain
/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    // NOTE We don't need an rcc parameter because it's enabled by default
    fn split(self) -> Self::Parts;
}

trait GpioRegExt {
    fn is_low(&self, pos: u8) -> bool;
    fn is_set_low(&self, pos: u8) -> bool;
    fn set_high(&self, pos: u8);
    fn set_low(&self, pos: u8);
}

pub struct AF0;
pub struct AF1;
pub struct AF2;
pub struct AF3;
pub struct AF4;
pub struct AF5;
pub struct AF6;
pub struct AF7;

pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;

/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

use embedded_hal::digital::{toggleable, InputPin, OutputPin, StatefulOutputPin};

macro_rules! gpio_trait {
    ($gpiox:ident) => {
        impl GpioRegExt for crate::xmc1100::$gpiox::RegisterBlock {
            fn is_low(&self, pos: u8) -> bool {
                self.in_.read().bits() & (1 << pos) != 0
            }

            fn is_set_low(&self, pos: u8) -> bool {
                self.out.read().bits() & (1 << pos) == 0
            }

            fn set_high(&self, pos: u8) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { self.omr.write(|w| w.bits(1 << (pos + 16))) }
            }

            fn set_low(&self, pos: u8) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { self.omr.write(|w| w.bits(1 << pos)) }
            }
        }
    };
}

gpio_trait!(port0);
gpio_trait!(port1);

#[allow(unused)]
macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty, $iocrx:ident, $pcx:ident),)+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use core::marker::PhantomData;

            use crate::xmc1100::$GPIOX;
            use embedded_hal::digital::{toggleable, InputPin, OutputPin, StatefulOutputPin};

            use cortex_m::interrupt::CriticalSection;

            use super::{
                Floating, GpioExt, Input, OpenDrain, Output,
                PullDown, PullUp, PushPull,
                Alternate, AF0, AF1, AF2, AF3, AF4, AF5, AF6, AF7,
                GpioRegExt,
            };

            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi<$MODE>,
                )+
            }

            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    Parts {
                        $(
                            $pxi: $PXi { _mode: PhantomData },
                        )+
                    }
                }
            }

            $(
                /// Pin
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }

                impl<MODE> $PXi<MODE> {
                    /// Configures the pin to operate as a floating input pin
                    pub fn into_floating_input(
                        self,
                        _cs: &CriticalSection
                    ) -> $PXi<Input<Floating>> {
                        unsafe {
                            &(*$GPIOX::ptr()).$iocrx.modify(|_, w| {
                                w.$pcx().value1()
                            });
                        }
                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled down input pin
                    pub fn into_pull_down_input(
                        self,
                        _cs: &CriticalSection
                        ) -> $PXi<Input<PullDown>> {
                        unsafe {
                            &(*$GPIOX::ptr()).$iocrx.modify(|_, w| {
                                w.$pcx().value2()
                            });
                        }
                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled up input pin
                    pub fn into_pull_up_input(
                        self,
                        _cs: &CriticalSection
                    ) -> $PXi<Input<PullUp>> {
                        unsafe {
                            &(*$GPIOX::ptr()).$iocrx.modify(|_, w| {
                                w.$pcx().value3()
                            });
                        }
                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an open drain output pin
                    pub fn into_open_drain_output(
                        self,
                        _cs: &CriticalSection
                    ) -> $PXi<Output<OpenDrain>> {
                        unsafe {
                            &(*$GPIOX::ptr()).$iocrx.modify(|_, w| {
                                w.$pcx().value17()
                            });
                        }
                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an push pull output pin
                    pub fn into_push_pull_output(
                        self,
                        _cs: &CriticalSection
                    ) -> $PXi<Output<PushPull>> {
                        unsafe {
                            &(*$GPIOX::ptr()).$iocrx.modify(|_, w| {
                                w.$pcx().value9()
                            });
                        }
                        $PXi { _mode: PhantomData }
                    }

                    // TODO This always configures it as PushPull
                    fn set_alternate_mode(&mut self, mode: u8) {
                        debug_assert!(mode < 0b1000);
                        unsafe {
                            &(*$GPIOX::ptr()).$iocrx.modify(|_, w| {
                                w.$pcx().bits(0b10000 | mode)
                            });
                        }
                    }

                    pub fn into_alternate_af0(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF0>> {
                        self.set_alternate_mode(0);
                        $PXi { _mode: PhantomData }
                    }
                    pub fn into_alternate_af1(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF1>> {
                        self.set_alternate_mode(1);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af2(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF2>> {
                        self.set_alternate_mode(2);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af3(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF3>> {
                        self.set_alternate_mode(3);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af4(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF4>> {
                        self.set_alternate_mode(4);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af5(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF5>> {
                        self.set_alternate_mode(5);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af6(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF6>> {
                        self.set_alternate_mode(6);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af7(
                        mut self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF7>> {
                        self.set_alternate_mode(7);
                        $PXi { _mode: PhantomData }
                    }

                }

                impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                    fn is_set_high(&self) -> bool {
                        !self.is_set_low()
                    }

                    fn is_set_low(&self) -> bool {
                        unsafe { (*$GPIOX::ptr()).is_set_low($i) }
                    }
                }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    fn set_high(&mut self) {
                        unsafe { (*$GPIOX::ptr()).set_high($i) }
                    }

                    fn set_low(&mut self) {
                        unsafe { (*$GPIOX::ptr()).set_low($i) }
                    }
                }

                impl<MODE> toggleable::Default for $PXi<Output<MODE>> {}

                impl InputPin for $PXi<Output<OpenDrain>> {
                    fn is_high(&self) -> bool {
                        !self.is_low()
                    }

                    fn is_low(&self) -> bool {
                        unsafe { (*$GPIOX::ptr()).is_low($i) }
                    }
                }

                impl<MODE> InputPin for $PXi<Input<MODE>> {
                    fn is_high(&self) -> bool {
                        !self.is_low()
                    }

                    fn is_low(&self) -> bool {
                        unsafe { (*$GPIOX::ptr()).is_low($i) }
                    }
                }
            )+
        }
    }
}

gpio!(PORT0, port0, [
    P0_0: (p0_0, 0, Input<Floating>, iocr0, pc0),
    P0_1: (p0_1, 1, Input<Floating>, iocr0, pc1),
    P0_2: (p0_2, 2, Input<Floating>, iocr0, pc2),
    P0_3: (p0_3, 3, Input<Floating>, iocr0, pc3),
    P0_4: (p0_4, 4, Input<Floating>, iocr4, pc4),
    P0_5: (p0_5, 5, Input<Floating>, iocr4, pc5),
    P0_6: (p0_6, 6, Input<Floating>, iocr4, pc6),
    P0_7: (p0_7, 7, Input<Floating>, iocr4, pc7),
    P0_8: (p0_8, 8, Input<Floating>, iocr8, pc8),
    P0_9: (p0_9, 9, Input<Floating>, iocr8, pc9),
    P0_10: (p0_10, 10, Input<Floating>, iocr8, pc10),
    P0_11: (p0_11, 11, Input<Floating>, iocr8, pc11),
    P0_12: (p0_12, 12, Input<Floating>, iocr12, pc12),
    P0_13: (p0_13, 13, Input<Floating>, iocr12, pc13),
    P0_14: (p0_14, 14, Input<Floating>, iocr12, pc14),
    P0_15: (p0_15, 15, Input<Floating>, iocr12, pc15),
]);

gpio!(PORT1, port1, [
    P1_0: (p1_0, 0, Input<Floating>, iocr0, pc0),
    P1_1: (p1_1, 1, Input<Floating>, iocr0, pc1),
    P1_2: (p1_2, 2, Input<Floating>, iocr0, pc2),
    P1_3: (p1_3, 3, Input<Floating>, iocr0, pc3),
    P1_4: (p1_4, 4, Input<Floating>, iocr4, pc4),
    P1_5: (p1_5, 5, Input<Floating>, iocr4, pc5),
    P1_6: (p1_6, 6, Input<Floating>, iocr4, pc6),
]);
