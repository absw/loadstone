//! Time units.
use core::ops::{Add as Adds, Sub as Subtracts};

/// Abstract point in time. Useful for time periods
///
/// Any implementer of Instant can be subtracted with
/// itself to obtain a span of milliseconds.
///
/// Any implementer of Instant can be added with
/// milliseconds to obtain another instant.
pub trait Instant
where
    Self: Copy + Clone,
    Self: Subtracts<Output = Milliseconds>,
    Self: Adds<Milliseconds, Output = Self>,
{
}

pub trait Now {
    type I: Instant;
    fn now(&self) -> Self::I;
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub struct Microseconds(pub u32);

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub struct Milliseconds(pub u32);

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub struct Seconds(pub u32);

/// Bits per second
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub struct Bps(pub u32);

/// Hertz
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub struct Hertz(pub u32);

/// KiloHertz
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub struct KiloHertz(pub u32);

/// MegaHertz
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub struct MegaHertz(pub u32);

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn bps(self) -> Bps;

    /// Wrap in `Hertz`
    fn hz(self) -> Hertz;

    /// Wrap in `KiloHertz`
    fn khz(self) -> KiloHertz;

    /// Wrap in `MegaHertz`
    fn mhz(self) -> MegaHertz;

    /// Wrap in `Seconds`
    fn s(self) -> Seconds;

    /// Wrap in `Milliseconds`
    fn ms(self) -> Milliseconds;

    /// Wrap in `Microseconds`
    fn us(self) -> Microseconds;
}

impl U32Ext for u32 {
    fn bps(self) -> Bps { Bps(self) }

    fn hz(self) -> Hertz { Hertz(self) }

    fn khz(self) -> KiloHertz { KiloHertz(self) }

    fn mhz(self) -> MegaHertz { MegaHertz(self) }

    fn s(self) -> Seconds { Seconds(self) }

    fn ms(self) -> Milliseconds { Milliseconds(self) }

    fn us(self) -> Microseconds { Microseconds(self) }
}

impl Into<Hertz> for KiloHertz {
    fn into(self) -> Hertz { Hertz(self.0 * 1_000) }
}

impl Into<Hertz> for MegaHertz {
    fn into(self) -> Hertz { Hertz(self.0 * 1_000_000) }
}

impl Into<KiloHertz> for MegaHertz {
    fn into(self) -> KiloHertz { KiloHertz(self.0 * 1_000) }
}

impl Into<Milliseconds> for Seconds {
    fn into(self) -> Milliseconds { Milliseconds(self.0 * 1_000) }
}

impl Into<Microseconds> for Seconds {
    fn into(self) -> Microseconds { Microseconds(self.0 * 1_000_000) }
}

impl Into<Microseconds> for Milliseconds {
    fn into(self) -> Microseconds { Microseconds(self.0 * 1_000) }
}
