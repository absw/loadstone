alternate_functions!(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,);
pin_rows!(a, b, c, d, e, f, g, h, i, j, k,);

#[cfg(feature = "stm32f429")]
mod pins {
    gpio!(b, [
       (7, Output::<PushPull>),
    ]);

    gpio!(d, [
        (5, AF7),
        (6, AF7),
    ]);
}

#[cfg(feature = "stm32f469")]
mod pins {
    gpio!(b, [
       (10, AF7),
       (11, AF7),
    ]);
    gpio!(d, [
        (4, Output<PushPull>),
    ]);
}

#[cfg(feature = "stm32f407")]
mod pins {
    gpio!(a, [
      (2, AF7),
      (3, AF7),
    ]);
    gpio!(d, [
      (14, Output<PushPull>),
    ]);
}

// Reexport facade mod
pub use pins::*;
