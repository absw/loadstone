alternate_functions!(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,);
pin_rows!(a, b, c, d, e, f, g, h, i, j, k,);

#[cfg(feature = "stm32f429")]
gpio!(b, [
   (7, Output::<PushPull>),
]);

#[cfg(feature = "stm32f429")]
gpio!(d, [
    (5, AF7),
    (6, AF7),
]);

#[cfg(feature = "stm32f469")]
gpio!(b, [
   (10, AF7),
   (11, AF7),
]);

#[cfg(feature = "stm32f469")]
gpio!(d, [
    (4, Output<PushPull>),
]);





