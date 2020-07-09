use crate::{
    hal::time::{Hertz, MegaHertz},
    stm32pac::{FLASH, RCC},
};

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    hclk: Hertz,
    pclk1: Hertz,
    pclk2: Hertz,
    sysclk: Hertz,
}

impl Clocks {
    pub fn hclk(&self) -> Hertz { self.hclk }

    pub fn pclk1(&self) -> Hertz { self.pclk1 }

    pub fn pclk2(&self) -> Hertz { self.pclk2 }

    pub fn sysclk(&self) -> Hertz { self.sysclk }

    /// Harcoded values for the f412
    #[cfg(feature = "stm32f412")]
    pub fn hardcoded(flash: FLASH, rcc: RCC) -> Self {
        // NOTE(Safety): All unsafe blocks in this function refer to using the "bits()"
        // method for easy writing.
        flash.acr.write(|w| {
            unsafe { w.latency().bits(1) }; // 50Mhz -> 1 wait state at 3.3v
            w.prften().set_bit()
        });

        rcc.cr.modify(|_, w| w.hseon().set_bit());
        while rcc.cr.read().hserdy().bit_is_clear() {}

        rcc.pllcfgr.write(|w| unsafe {
            w.pllsrc().set_bit(); // HSE input to PLL
            w.pllm().bits(8);
            w.plln().bits(100);
            w.pllp().bits(0); // pllp = (divider / 2) >> 1
            w.pllq().bits(3)
        });

        rcc.cr.modify(|_, w| w.pllon().set_bit());
        while rcc.cr.read().pllrdy().bit_is_clear() {}

        rcc.cfgr.modify(|_, w| unsafe {
            w.ppre1().bits(0b100); // Divided by 2
            w.ppre2().bits(0b000); // Divided by 1
            w.hpre().bits(0b000); // Divided by 1
            w.sw().bits(0b10) // PLL source
        });

        while rcc.cfgr.read().sws().bits() != 0b10 {}
        Self {
            hclk: MegaHertz(50).into(),
            pclk1: MegaHertz(25).into(),
            pclk2: MegaHertz(50).into(),
            sysclk: MegaHertz(50).into(),
        }
    }
}
