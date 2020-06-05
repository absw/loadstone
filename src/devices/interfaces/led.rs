use crate::hal::time;

/// Interface to a LED's generic color
pub trait Chromatic<Color> {
    fn color(&mut self, color: Color);
}

pub trait Toggle {
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}

pub trait Blink {
    fn frequency(&mut self, frequency: time::Hertz);
}
