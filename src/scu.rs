//! Rcc Handling
use xmc1100::SCU_CLK;
use xmc1100::SCU_GENERAL;

// TODO This is mostly a shim. Port more from stm32f0xx-hal

use crate::time::{Hertz, MegaHertz};

/// Constrained SCU peripheral
pub struct Scu {
    pub clocks: Clocks,
    pub(crate) scu_general: SCU_GENERAL,
    pub(crate) scu_clk: SCU_CLK,
}

impl Scu {
    pub fn new(scu_general: SCU_GENERAL, scu_clk: SCU_CLK) -> Self {
        // Disable write protection
        scu_general
            .passwd
            .write(|w| w.pass().value1().mode().value1());
        Scu {
            clocks: Clocks {
                sysclk: MegaHertz(8).into(),
            },
            scu_general,
            scu_clk,
        }
    }
}
/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    sysclk: Hertz,
}

impl Clocks {
    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
