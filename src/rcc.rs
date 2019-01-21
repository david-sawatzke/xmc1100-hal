//! Rcc Handling

// TODO This is mostly a shim. Port more from stm32f0xx-hal

use crate::time::Hertz;

/// Constrained RCC peripheral
pub struct Rcc {
    pub clocks: Clocks,
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    pub sysclk: Hertz,
}

impl Clocks {
    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
