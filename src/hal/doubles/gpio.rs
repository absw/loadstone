use crate::hal::gpio::OutputPin;
use std::vec::Vec;

#[derive(Clone, Debug, Default)]
pub struct MockPin {
    pub state: bool,
    pub changes: Vec<bool>,
}

impl MockPin {
    pub fn is_high(&self) -> bool { self.state }
    pub fn is_low(&self) -> bool { !self.state }
}

impl OutputPin for MockPin {
    fn set_low(&mut self) {
        self.state = false;
        self.changes.push(self.state);
    }

    fn set_high(&mut self) {
        self.state = true;
        self.changes.push(self.state);
    }
}
