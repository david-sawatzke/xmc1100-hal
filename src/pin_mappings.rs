use crate::gpio::port0::*;
use crate::gpio::port1::*;
use crate::gpio::port2::*;
use crate::gpio::*;
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
