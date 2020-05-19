pub trait OutputPin {
    fn set_low(&mut self);
    fn set_high(&mut self);
}

pub trait InputPin {
    fn is_high(&self) -> bool;
    fn is_low(&self) -> bool;
}
