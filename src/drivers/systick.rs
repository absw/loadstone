use crate::hal::time;

/// Opaque wrapper around a system tick at certain point in time
#[derive(Copy, Clone, Debug)]
pub struct Tick {
    counter: u32,
    sysclk_frequency: time::Hertz,
}

/// Tick subtraction to obtain a time period
impl core::ops::Sub for Tick {
    type Output = time::Milliseconds;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.sysclk_frequency == rhs.sysclk_frequency);
        let difference = self.counter.wrapping_sub(rhs.counter);
        time::Milliseconds( (difference * 1000u32) / self.sysclk_frequency.0 )
    }
}

/// Addition between any Millisecond-convertible type and the current tick.
impl<T: Into<time::Milliseconds>> core::ops::Add<T> for Tick {
    type Output = Self;

    fn add(self, rhs: T) -> Self {
        Self {
            counter: self.counter + ((rhs.into().0 * self.sysclk_frequency.0) / 1000u32),
            sysclk_frequency: self.sysclk_frequency
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
        let test_tick_late = Tick { counter: test_tick_early.counter + ticks_difference, sysclk_frequency };

        // Then (1000 ticks at 2000 hertz)
        assert_eq!(time::Milliseconds(500), test_tick_late - test_tick_early);

        // Given
        let test_tick_late = test_tick_late + time::Milliseconds(300);

        // Then (1000 ticks at 2000 hertz + 300 milliseconds)
        assert_eq!(time::Milliseconds(800), test_tick_late - test_tick_early);
    }
}
