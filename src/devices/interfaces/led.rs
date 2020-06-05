use crate::hal::time;

/// Interface to a LED's generic color
pub trait Chromatic<Color> {
    fn color(self, color: Color) -> Self;
}

pub trait Toggle {
    fn on(self) -> Self;
    fn off(self) -> Self;
    fn toggle(self) -> Self;
}

pub trait Blink {
    fn frequency(self, frequency: time::Hertz) -> Self;
}
