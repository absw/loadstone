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
    use crate::drivers::serial::UsartAf;
    gpio!(b, [(2, AF9),]);
    gpio!(f, [(6, AF9), (7, AF9), (8, AF10), (9, AF10),]);
    gpio!(g, [(6, AF10), (14, super::UsartAf), (9, super::UsartAf),]);
}

// Reexport facade mod
pub use pins::*;
