gpio!(GPIOB, gpiob, gpiob, gpioben, gpiobrst, PBx, [
      PB7: (pb7, 7, Output<PushPull>), [],
]);

gpio!(GPIOD, gpiod, gpiok, gpioden, gpiodrst, PDx, [
      PD5: (pd5, 5, AF7), [Usart2TxPin],
      PD6: (pd6, 6, AF7), [Usart2RxPin],
]);
