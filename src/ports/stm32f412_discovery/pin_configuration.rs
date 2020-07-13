//! GPIO configuration and alternate functions for the [stm32f412 discovery](../../../../../../documentation/hardware/discovery.pdf).
pin_rows!(a, b, c, d, e, f, g, h, i, j, k,);
mod pins {
    gpio!(e, [(1, Output<PushPull>),]);
    gpio!(b, [(2, AF9),]);
    gpio!(f, [(6, AF9), (7, AF9), (8, AF10), (9, AF10),]);
    gpio!(g, [(6, AF10), (14, AF8), (9, AF8),]);
}
pub use pins::*;
