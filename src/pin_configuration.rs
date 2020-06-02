alternate_functions!(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,);
pin_rows!(a, b, c, d, e, f, g, h, i, j, k,);

gpio!(b, [
   (7, Output::<PushPull>),
]);

gpio!(d, [
    (5, AF7),
    (6, AF7),
]);
