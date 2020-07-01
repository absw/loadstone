//! GPIO configuration and alternate functions.

pin_rows!(a, b, c, d, e, f, g, h, i, j, k,);

#[cfg(feature = "stm32f429")]
mod pins {
    gpio!(b, [(7, Output::<PushPull>),]);
}

#[cfg(feature = "stm32f469")]
mod pins {
    gpio!(d, [
        (4, Output<PushPull>),
    ]);
}

#[cfg(feature = "stm32f407")]
mod pins {
    gpio!(d, [
      (14, Output<PushPull>),
    ]);
}

#[cfg(feature = "stm32f412")]
mod pins {
    use crate::drivers::spi::SpiAf;
    gpio!(a, [
       (5, super::SpiAf),
       (6, super::SpiAf),
       (7, super::SpiAf),
    ]);
    gpio!(g, [(6, Output::<PushPull>),]);
}

// Reexport facade mod
pub use pins::*;
