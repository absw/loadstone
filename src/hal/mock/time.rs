use crate::hal::time;

#[derive(Copy, Clone, Debug)]
pub struct MockInstant {}
pub struct MockSysTick {}

impl time::Instant for MockInstant {}

impl time::Now<MockInstant> for MockSysTick {
    fn now(&self) -> MockInstant { MockInstant {} }
}

impl core::ops::Sub for MockInstant {
    type Output = time::Milliseconds;
    fn sub(self, rhs: Self) -> Self::Output { time::Milliseconds(0) }
}

/// Addition between any Millisecond-convertible type and the current tick.
impl<T: Into<time::Milliseconds>> core::ops::Add<T> for MockInstant {
    type Output = Self;
    fn add(self, rhs: T) -> Self { Self{} }
}
