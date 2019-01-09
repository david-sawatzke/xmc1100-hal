pub use embedded_hal::prelude::*;
// TODO for some reason, watchdog isn't in the embedded_hal prelude
pub use crate::gpio::GpioExt as _xmc1100_hal_gpio_GpioExt;
