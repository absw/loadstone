//! GPIO configuration and alternate functions.

pin_rows!(a, b, c, d, e, f, g, h, i, j, k,);

#[cfg(feature = "stm32f429")]
mod pins {
    gpio!(b, [(7, Output::<PushPull>),]);

    gpio!(d, [(5, AF7), (6, AF7),]);
}

#[cfg(feature = "stm32f469")]
mod pins {
    gpio!(b, [(10, AF7), (11, AF7),]);
    gpio!(d, [
        (4, Output<PushPull>),
    ]);
}

#[cfg(feature = "stm32f407")]
mod pins {
    gpio!(a, [(2, AF7), (3, AF7),]);
    gpio!(d, [
      (14, Output<PushPull>),
    ]);
}

// Reexport facade mod
pub use pins::*;
