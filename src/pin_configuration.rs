gpio!(GPIOB, gpiob, gpiob, gpioben, gpiobrst, PBx, [
      PB7: (pb7, 7, Output<PushPull>, AFRL), [],
]);

gpio!(GPIOD, gpiod, gpiok, gpioden, gpiodrst, PDx, [
      PD8: (pd8, 8, AF7, AFRH), [Usart3TxPin],
      PD9: (pd9, 9, AF7, AFRH), [Usart3RxPin],
]);
