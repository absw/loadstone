//! Various LED device implementations.

use crate::{
    devices::interfaces::led::{self, Chromatic, Toggle},
    hal::gpio::OutputPin,
};

/// Multi-color type for RGB LEDs
#[derive(Copy, Clone, Debug, is_enum_variant)]
pub enum RgbPalette {
    Red,
    Green,
    Blue,
}

/// Solid (non-blinking) color RGB LED
///
/// # Example
/// ```
/// # use secure_bootloader_lib::devices::implementations::led::*;
/// # use secure_bootloader_lib::devices::interfaces::led::*;
/// # use secure_bootloader_lib::hal::mock::gpio::*;
/// # let pin = MockPin::default();
/// # let (red_pin, green_pin, blue_pin) = (pin.clone(), pin.clone(), pin.clone());
/// let mut led = RgbLed::new(red_pin, green_pin, blue_pin, LogicLevel::Direct);
///
/// // By default, the LED starts on an off, green state
/// # assert!(led.pin(RgbPalette::Red).is_low());
/// # assert!(led.pin(RgbPalette::Green).is_low());
/// # assert!(led.pin(RgbPalette::Blue).is_low());
/// assert!(!led.is_on());
///
/// led.on(); // This will shine green
/// # assert!(led.pin(RgbPalette::Red).is_low());
/// # assert!(led.pin(RgbPalette::Green).is_high());
/// # assert!(led.pin(RgbPalette::Blue).is_low());
/// assert!(led.get_color().is_green());
/// assert!(led.is_on());
///
/// led.color(RgbPalette::Blue);
/// led.toggle();
/// # assert!(led.pin(RgbPalette::Red).is_low());
/// # assert!(led.pin(RgbPalette::Green).is_low());
/// # assert!(led.pin(RgbPalette::Blue).is_low());
/// assert!(led.get_color().is_blue());
/// assert!(!led.is_on());
/// ```
pub struct RgbLed<Pin: OutputPin> {
    red: Pin,
    green: Pin,
    blue: Pin,
    color: RgbPalette,
    is_on: bool,
    logic: LogicLevel,
}

/// Solid (non-blinking) monochrome LED
///
/// # Example
/// ```
/// # use secure_bootloader_lib::devices::implementations::led::*;
/// # use secure_bootloader_lib::devices::interfaces::led::*;
/// # use secure_bootloader_lib::hal::mock::gpio::*;
/// # let pin = MockPin::default();
/// let mut led = MonochromeLed::new(pin, LogicLevel::Direct);
///
/// led.toggle();
/// assert!(led.is_on());
/// # assert!(led.pin().is_high());
/// ```
pub struct MonochromeLed<Pin: OutputPin> {
    pin: Pin,
    is_on: bool,
    logic: LogicLevel,
}

#[derive(Copy, Clone)]
pub enum LogicLevel {
    /// LogicLevelal high equals "on"
    Direct,
    /// LogicLevelal high equals "off"
    Inverted,
}

// Extension trait to ensure LED pins are correctly
// operated based on the led's direct or inverted logic
trait LedPin: OutputPin {
    fn off(&mut self, logic: LogicLevel) {
        if let LogicLevel::Direct = logic {
            self.set_low();
        } else {
            self.set_high();
        }
    }

    fn on(&mut self, logic: LogicLevel) {
        if let LogicLevel::Direct = logic {
            self.set_high();
        } else {
            self.set_low();
        }
    }
}

// Blanket implementation of LedPin for all output pins
impl<Pin: OutputPin> LedPin for Pin {}

impl<Pin: OutputPin> MonochromeLed<Pin> {
    pub fn new(mut pin: Pin, logic: LogicLevel) -> Self {
        pin.off(logic);
        Self { pin, is_on: false, logic }
    }
    pub fn is_on(&self) -> bool { self.is_on }
}

// Turn off when going out of scope
impl<Pin: OutputPin> Drop for MonochromeLed<Pin> {
    fn drop(&mut self) { self.off(); }
}

impl<Pin: OutputPin> led::Toggle for MonochromeLed<Pin> {
    fn on(&mut self) {
        if !self.is_on {
            self.pin.on(self.logic);
        }
        self.is_on = true;
    }

    fn off(&mut self) {
        if self.is_on {
            self.pin.off(self.logic);
        }
        self.is_on = false;
    }

    fn toggle(&mut self) {
        if self.is_on {
            self.off();
        } else {
            self.on();
        }
    }
}

impl<Pin: OutputPin> RgbLed<Pin> {
    pub fn new(mut red: Pin, mut green: Pin, mut blue: Pin, logic: LogicLevel) -> Self {
        red.off(logic);
        green.off(logic);
        blue.off(logic);
        Self { red, green, blue, color: RgbPalette::Green, is_on: false, logic }
    }
    pub fn get_color(&self) -> RgbPalette { self.color }
    pub fn is_on(&self) -> bool { self.is_on }
}

// Turn off when going out of scope
impl<Pin: OutputPin> Drop for RgbLed<Pin> {
    fn drop(&mut self) { self.off(); }
}

impl<Pin: OutputPin> led::Toggle for RgbLed<Pin> {
    fn on(&mut self) {
        if !self.is_on {
            match self.color {
                RgbPalette::Red => {
                    self.red.on(self.logic);
                }
                RgbPalette::Green => {
                    self.green.on(self.logic);
                }
                RgbPalette::Blue => {
                    self.blue.on(self.logic);
                }
            }
        }
        self.is_on = true;
    }

    fn off(&mut self) {
        if self.is_on {
            self.red.off(self.logic);
            self.green.off(self.logic);
            self.blue.off(self.logic);
        }
        self.is_on = false;
    }

    fn toggle(&mut self) {
        if self.is_on {
            self.off();
        } else {
            self.on();
        }
    }
}

impl<Pin: OutputPin> Chromatic<RgbPalette> for RgbLed<Pin> {
    fn color(&mut self, color: RgbPalette) {
        self.color = color;
        if self.is_on {
            self.off();
            self.on();
        }
    }
}

#[cfg(not(target_arch = "arm"))]
#[doc(hidden)]
pub mod mock {
    use super::*;
    use crate::hal::mock::gpio::*;

    #[doc(hidden)]
    impl MonochromeLed<MockPin> {
        pub fn pin(&self) -> &MockPin { &self.pin }
    }

    #[doc(hidden)]
    impl RgbLed<MockPin> {
        pub fn pin(&self, color: RgbPalette) -> &MockPin {
            match color {
                RgbPalette::Red => &self.red,
                RgbPalette::Green => &self.green,
                RgbPalette::Blue => &self.blue,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{mock::*, *};
    use crate::hal::mock::gpio::*;

    #[test]
    fn monochrome_led_defaults_to_logic_low_with_direct_logic() {
        // Given
        let led = MonochromeLed::new(MockPin::default(), LogicLevel::Direct);

        // then
        assert!(led.pin.is_low());
    }

    #[test]
    fn monochrome_led_defaults_to_logic_high_with_inverted_logic() {
        // Given
        let led = MonochromeLed::new(MockPin::default(), LogicLevel::Inverted);

        // then
        assert!(led.pin.is_high());
    }

    #[test]
    fn monochrome_pin_setting() {
        // Given
        let mut led = MonochromeLed::new(MockPin::default(), LogicLevel::Direct);

        // When
        led.off();

        // Then
        assert!(led.pin.is_low());

        // When
        led.on();

        // Then
        assert!(led.pin.is_high());
    }

    #[test]
    fn monochrome_pin_toggling() {
        // Given
        let mut led = MonochromeLed::new(MockPin::default(), LogicLevel::Direct);

        // When
        led.toggle();

        // Then
        assert!(led.pin.is_high());

        // When
        led.toggle();

        // Then
        assert!(led.pin.is_low());
    }

    #[test]
    fn type_erasure_between_chromatic_and_non_chromatic_led() {
        // Given
        let mut monochrome = MonochromeLed::new(MockPin::default(), LogicLevel::Inverted);
        let mut chromatic =
            RgbLed::new(MockPin::default(), MockPin::default(), MockPin::default(), LogicLevel::Direct);

        chromatic.color(RgbPalette::Red);

        // Array over generic "toggleable" leds
        let mut array: [&mut dyn led::Toggle; 2] = [&mut monochrome, &mut chromatic];

        // When
        array.iter_mut().for_each(|l| l.toggle());

        // Then
        assert!(monochrome.pin.is_low()); // Inverted
        assert!(chromatic.red.is_high());
        assert!(chromatic.green.is_low());
        assert!(chromatic.blue.is_low());
    }
}
