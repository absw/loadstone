//! LED interfaces
//!
//! Access to LEDs is segmented over three interfaces to facilitate
//! all usual LED patterns.

use crate::hal::time;

/// Interface to a LED's generic color. May be tricolor LEDs, full color
/// scales with PWM, a "grayscale" intensity range, etc.
pub trait Chromatic<Color> {
    fn color(&mut self, color: Color);
}

/// Interface to a LED's direct on/off/toggle operations. Likely to be
/// implemented for all LEDs, but could be left off for a blinking LED
/// that must always remain on.
pub trait Toggle {
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}

/// Interface to a blink-capable LED.
pub trait Blink {
    fn frequency(&mut self, frequency: time::Hertz);
}
