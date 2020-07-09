use crate::{drivers::rcc, hal::time};
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::exception;
use time::Now;

/// Opaque wrapper around a system tick at certain point in time
#[derive(Copy, Clone, Debug)]
pub struct Tick {
    counter: u32,
    sysclk_frequency: time::Hertz,
}

impl time::Instant for Tick {}

/// Handle over the SysTick. Allows safe access to the current instant.
///
/// Existence of this type (or any copy) guarantees the systick peripheral
/// has been configured.
#[derive(Copy, Clone, Debug)]
pub struct SysTick {
    clocks: rcc::Clocks,
}

impl SysTick {
    /// Consumes the systick peripheral.
    pub fn new(mut systick: SYST, clocks: rcc::Clocks) -> Self {
        systick.set_clock_source(SystClkSource::Core);
        systick.set_reload(clocks.sysclk().0);
        systick.clear_current();
        systick.enable_counter();
        systick.enable_interrupt();
        Self { clocks }
    }

    pub fn wait<T: Copy + Into<time::Milliseconds>>(&self, t: T) {
        let start = self.now();
        while self.now() - start < t.into() {}
    }
}

impl Now<Tick> for SysTick {
    fn now(&self) -> Tick {
        let counter = TICK_COUNTER.load(Ordering::Relaxed);
        Tick {
            counter,
            sysclk_frequency: self.clocks.sysclk(),
        }
    }
}

static TICK_COUNTER: AtomicU32 = AtomicU32::new(0);

#[exception]
fn SysTick() {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Tick subtraction to obtain a time period
impl core::ops::Sub for Tick {
    type Output = time::Milliseconds;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.sysclk_frequency == rhs.sysclk_frequency);
        let difference = self.counter.wrapping_sub(rhs.counter);
        time::Milliseconds((difference * 1000u32) / self.sysclk_frequency.0)
    }
}

/// Addition between any Millisecond-convertible type and the current tick.
impl<T: Into<time::Milliseconds>> core::ops::Add<T> for Tick {
    type Output = Self;

    fn add(self, rhs: T) -> Self {
        Self {
            counter: self.counter + ((rhs.into().0 * self.sysclk_frequency.0) / 1000u32),
            sysclk_frequency: self.sysclk_frequency,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tick_differences_and_additions() {
        // Given
        let sysclk_frequency = time::Hertz(2000);
        let ticks_difference = 1000u32;
        let test_tick_early = Tick { counter: 0, sysclk_frequency };
        let test_tick_late =
            Tick { counter: test_tick_early.counter + ticks_difference, sysclk_frequency };

        // Then (1000 ticks at 2000 hertz)
        assert_eq!(time::Milliseconds(500), test_tick_late - test_tick_early);

        // Given
        let test_tick_late = test_tick_late + time::Milliseconds(300);

        // Then (1000 ticks at 2000 hertz + 300 milliseconds)
        assert_eq!(time::Milliseconds(800), test_tick_late - test_tick_early);
    }
}
