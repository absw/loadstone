use crate::devices::interfaces::led::{self, Toggle, Chromatic};
use crate::hal::gpio::OutputPin;

#[derive(Copy, Clone, Debug)]
/// Multi-color type for RGB LEDs
pub enum RgbPalette {
    Red,
    Green,
    Blue,
}

/// Solid (non-blinking) color RGB LED
///
/// Implements Chromatic, Toggle and Blink
pub struct RgbLed<Pin: OutputPin> {
    red: Pin,
    green: Pin,
    blue: Pin,
    color: RgbPalette,
    is_on: bool,
    logic: Logic,
}

/// Solid (non-blinking) monochrome LED
///
/// Implements Toggle and Blink
pub struct MonochromeLed<Pin: OutputPin> {
    pin: Pin,
    is_on: bool,
    logic: Logic,
}

#[derive(Copy, Clone)]
pub enum Logic {
    /// Logical high equals "on"
    Direct,
    /// Logical high equals "off"
    Inverted
}

// Extension trait to ensure LED pins are correctly
// operated based on the led's direct or inverted logic
trait LedPin: OutputPin {
    fn off(&mut self, logic: Logic) {
        if let Logic::Direct = logic {
            self.set_low();
        } else {
            self.set_high();
        }
    }

    fn on(&mut self, logic: Logic) {
        if let Logic::Direct = logic {
            self.set_high();
        } else {
            self.set_low();
        }
    }
}

// Blanket implementation of LedPin for all output pins
impl<Pin: OutputPin> LedPin for Pin {}

impl <Pin: OutputPin> led::Toggle for MonochromeLed<Pin> {
    fn on(mut self) -> Self {
        if !self.is_on { self.pin.on(self.logic); }
        self.is_on = true;
        self
    }

    fn off(mut self) -> Self {
        if self.is_on { self.pin.off(self.logic); }
        self.is_on = false;
        self
    }

    fn toggle(self) -> Self{
        if self.is_on { self.off() } else { self.on() }
    }
}

impl <Pin: OutputPin> led::Toggle for RgbLed<Pin> {
    fn on(mut self) -> Self {
        if !self.is_on {
            match self.color {
                RgbPalette::Red => { self.red.on(self.logic); },
                RgbPalette::Green => { self.green.on(self.logic); },
                RgbPalette::Blue => { self.blue.on(self.logic); },
            }
        }
        self.is_on = true;
        self
    }

    fn off(mut self) -> Self {
        if self.is_on {
            self.red.off(self.logic);
            self.green.off(self.logic);
            self.blue.off(self.logic);
        }
        self.is_on = false;
        self
    }

    fn toggle(self) -> Self {
        if self.is_on { self.off() } else { self.on() }
    }
}

impl<Pin: OutputPin> led::Chromatic<RgbPalette> for RgbLed<Pin> {
    fn color(mut self, color: RgbPalette) -> Self {
        self.color = color;
        if self.is_on {
            self.off().on()
        } else {
            self
        }
    }
}

impl<Pin: OutputPin> RgbLed<Pin> {
    pub fn new(red: Pin, green: Pin, blue: Pin, logic: Logic) -> Self {
        Self { red, green, blue, color: RgbPalette::Green, is_on: false, logic }
    }
    pub fn get_color(&self) -> RgbPalette { self.color }
    pub fn is_on(&self) -> bool { self.is_on }
}
