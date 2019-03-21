//! Rcc Handling
use xmc1100::SCU_CLK;
use xmc1100::SCU_GENERAL;

// TODO This is mostly a shim. Port more from stm32f0xx-hal

use crate::time::{Hertz, MegaHertz};

/// Constrained SCU peripheral
pub struct Scu {
    pub clocks: Clocks,
    pub(crate) _scu_general: SCU_GENERAL,
    pub(crate) scu_clk: SCU_CLK,
}

impl Scu {
    pub fn new(scu_general: SCU_GENERAL, scu_clk: SCU_CLK) -> ClockConfig {
        // Disable write protection
        scu_general
            .passwd
            .write(|w| w.pass().value1().mode().value1());
        let scu = Scu {
            clocks: Clocks {
                sysclk: MegaHertz(8).into(),
            },
            _scu_general: scu_general,
            scu_clk,
        };
        ClockConfig { scu, sysclk: None }
    }
}

pub struct ClockConfig {
    scu: Scu,
    sysclk: Option<u32>,
}

impl ClockConfig {
    pub fn sysclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sysclk = Some(freq.into().0);
        self
    }

    pub fn freeze(mut self) -> Scu {
        // TODO Do temperature calibration
        if let Some(sysclk) = self.sysclk {
            let idiv = 32000000 / sysclk;
            if idiv > 0xFF || idiv == 0 {
                panic!("Divider for sysclk invalid");
            }
            unsafe { self.scu.scu_clk.clkcr.write(|w| w.idiv().bits(idiv as u8)) };
            // Calculate real frequency
            self.scu.clocks.sysclk = Hertz(32000000 / idiv);
        } else {
            // Set default frequency of 8MHz
            unsafe { self.scu.scu_clk.clkcr.write(|w| w.idiv().bits(0x04)) };
        }
        self.scu
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
