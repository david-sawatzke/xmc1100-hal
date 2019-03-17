use crate::gpio::port0::*;
use crate::gpio::port1::*;
use crate::gpio::port2::*;
use crate::gpio::*;
use crate::usic::Dout0Pin;
use crate::usic::*;
use xmc1100::*;

macro_rules! pins {
    ($($PIN:ident => {
        $($AF:ty: $TRAIT:ty),+
    }),+) => {
        $(
            $(
                impl $TRAIT for $PIN<Alternate<$AF>> {}
            )+
        )+
    }
}

pins! {
    P0_6 => {AF7: Dout0Pin<USIC0_CH1>},
    P0_7 => {AF7: Dout0Pin<USIC0_CH1>},
    P0_14 => {AF6: Dout0Pin<USIC0_CH0>},
    P0_15 => {AF6: Dout0Pin<USIC0_CH0>},
    P1_0 => {AF7: Dout0Pin<USIC0_CH0>},
    P1_1 => {AF6: Dout0Pin<USIC0_CH0>},
    P1_2 => {AF7: Dout0Pin<USIC0_CH1>},
    P1_3 => {AF7: Dout0Pin<USIC0_CH1>},
    P1_5 => {AF2: Dout0Pin<USIC0_CH0>},
    P1_6 => {AF2: Dout0Pin<USIC0_CH1>},
    P2_0 => {AF6: Dout0Pin<USIC0_CH0>},
    P2_1 => {AF6: Dout0Pin<USIC0_CH0>},
    P2_10 => {AF7: Dout0Pin<USIC0_CH1>},
    P2_11 => {AF7: Dout0Pin<USIC0_CH1>}
}

macro_rules! input_pins_usic {
    ($($PIN:ident => {
        $($USIC:ident => $DxX:ident: $chan:expr),+
    }),+) => {
        $(
            $(
                impl<STATE> $DxX<$USIC> for $PIN<Input<STATE>> {
                    fn number() -> u8 {
                        $chan
                    }
                }
            )+
        )+
    }
}

input_pins_usic! {
    P0_0 => {
        USIC0_CH0 => Dx2Pin: 0,
        USIC0_CH1 => Dx2Pin: 0
    },
    P0_6 => {
        USIC0_CH1 => Dx0Pin: 2
    },
    P0_7 => {
        USIC0_CH0 => Dx1Pin: 2,
        USIC0_CH1 => Dx0Pin: 3,
        USIC0_CH1 => Dx1Pin: 2
    },
    P2_2 => {
        USIC0_CH0 => Dx3Pin: 0,
        USIC0_CH0 => Dx4Pin: 0,
        USIC0_CH0 => Dx5Pin: 0
    }
}
