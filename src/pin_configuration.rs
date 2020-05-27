gpio!(GPIOB, gpiob, gpioben, gpiobrst, PBx, [
      PB7: (pb7, 7, Output<PushPull>, AFRL), [],
      PA9: (pa9, 9, AF7, AFRH), [Usart1TxPin],
      PA10: (pa10, 10, AF7, AFRH), [Usart1RxPin],
]);
