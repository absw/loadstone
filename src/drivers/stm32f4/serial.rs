//! USART implementation.
use crate::{
    drivers::stm32f4::{gpio::*, rcc},
    hal::serial,
    ports::pin_configuration::*,
    stm32pac::{RCC, USART1, USART2, USART3, USART6},
};
use core::{marker::PhantomData, ptr};

/// Extension trait to wrap a USART peripheral into a more useful
/// high level abstraction.
pub trait UsartExt<PINS> {
    /// The wrapping type
    type Serial;

    fn constrain(
        self,
        pins: PINS,
        config: config::Config,
        clocks: rcc::Clocks,
    ) -> Result<Self::Serial, config::InvalidConfig>;
}

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// Sealed trait for all pins that can be TX for each USART.
/// This can't be implemented by the library user: All available
/// pins should already be implemented internally.
pub unsafe trait TxPin<USART>: private::Sealed {}

/// Sealed trait for all pins that can be RX for each USART.
/// This can't be implemented by the library user: All available
/// pins should already be implemented internally.
pub unsafe trait RxPin<USART>: private::Sealed {}

macro_rules! seal_pins { ($function:ty: [$($pin:ty,)+]) => {
    $(
        unsafe impl $function for $pin {}
        impl private::Sealed for $pin {}
    )+
};}

// List of all pins capable of being configured as certain USART
// functions. NOTE: This is not configuration! there's no need
// to remove items from these lists once complete.
#[cfg(any(feature = "stm32f469", feature = "stm32f429", feature = "stm32f407"))]
seal_pins!(TxPin<USART1>: [Pa9<AF7>, Pb6<AF7>,]);
#[cfg(any(feature = "stm32f412"))]
seal_pins!(TxPin<USART1>: [Pa9<AF7>, Pb6<AF7>, Pa15<AF6>,]);

#[cfg(any(feature = "stm32f469", feature = "stm32f429", feature = "stm32f407"))]
seal_pins!(RxPin<USART1>: [Pb7<AF7>, Pa10<AF7>,]);
#[cfg(any(feature = "stm32f412"))]
seal_pins!(RxPin<USART1>: [Pb3<AF7>, Pb7<AF7>, Pa10<AF7>,]);

#[cfg(any(
    feature = "stm32f469",
    feature = "stm32f429",
    feature = "stm32f407",
    feature = "stm32f412"
))]
seal_pins!(TxPin<USART2>: [Pa2<AF7>, Pd5<AF7>,]);

#[cfg(any(
    feature = "stm32f469",
    feature = "stm32f429",
    feature = "stm32f407",
    feature = "stm32f412"
))]
seal_pins!(RxPin<USART2>: [Pa3<AF7>, Pd6<AF7>,]);

#[cfg(any(feature = "stm32f412"))]
seal_pins!(TxPin<USART6>: [Pc6<AF8>, Pa11<AF8>, Pg14<AF8>,]);
#[cfg(any(feature = "stm32f412"))]
seal_pins!(RxPin<USART6>: [Pc7<AF8>, Pa12<AF8>, Pg9<AF8>,]);

/// Serial error
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
}

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
    /// Idle line state detected
    Idle,
}

pub mod config {
    //! Configuration required to construct a new USART instance.
    //!
    //! # Example
    //! ```ignore
    //! let (serial, tx, rx) = (peripherals.USART2, gpiod.pd5, gpiod.pd6);
    //! let serial_config = serial::config::Config::default().baudrate(Bps(115_200));
    //! let mut serial = serial.constrain((tx,rx), serial_config, clocks).unwrap();
    //! ```

    use crate::hal::time::{Bps, U32Ext};

    pub enum WordLength {
        DataBits8,
        DataBits9,
    }

    pub enum Parity {
        ParityNone,
        ParityEven,
        ParityOdd,
    }

    pub enum StopBits {
        #[doc = "1 stop bit"]
        STOP1,
        #[doc = "0.5 stop bits"]
        STOP0P5,
        #[doc = "2 stop bits"]
        STOP2,
        #[doc = "1.5 stop bits"]
        STOP1P5,
    }

    pub struct Config {
        pub baudrate: Bps,
        pub wordlength: WordLength,
        pub parity: Parity,
        pub stopbits: StopBits,
    }

    impl Config {
        pub fn baudrate(mut self, baudrate: Bps) -> Self {
            self.baudrate = baudrate;
            self
        }

        pub fn parity_none(mut self) -> Self {
            self.parity = Parity::ParityNone;
            self
        }

        pub fn parity_even(mut self) -> Self {
            self.parity = Parity::ParityEven;
            self
        }

        pub fn parity_odd(mut self) -> Self {
            self.parity = Parity::ParityOdd;
            self
        }

        pub fn wordlength_8(mut self) -> Self {
            self.wordlength = WordLength::DataBits8;
            self
        }

        pub fn wordlength_9(mut self) -> Self {
            self.wordlength = WordLength::DataBits9;
            self
        }

        pub fn stopbits(mut self, stopbits: StopBits) -> Self {
            self.stopbits = stopbits;
            self
        }
    }

    #[derive(Debug)]
    pub struct InvalidConfig;

    impl Default for Config {
        fn default() -> Config {
            let baudrate = 19_200_u32.bps();
            Config {
                baudrate,
                wordlength: WordLength::DataBits8,
                parity: Parity::ParityNone,
                stopbits: StopBits::STOP1,
            }
        }
    }
}

/// Marker trait for a tuple of pins that work for a given USART.
/// Automatically implemented for any tuple (A, B) where A is
/// a TxPin and B is a RxPin.
pub trait Pins<USART> {}

impl<USART, TX, RX> Pins<USART> for (TX, RX)
where
    TX: TxPin<USART>,
    RX: RxPin<USART>,
{
}

/// Serial abstraction
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

/// Serial receiver
pub struct Rx<USART> {
    _usart: PhantomData<USART>,
}

/// Serial transmitter
pub struct Tx<USART> {
    _usart: PhantomData<USART>,
}

macro_rules! hal_usart_impl {
    ($(
        $USARTX:ident: ($usartX:ident, $apbXenr:ident, $usartXen:ident,  $pclkX:ident),
    )+) => {
        $(
            impl<PINS> Serial<$USARTX, PINS> {
                pub fn $usartX(
                    usart: $USARTX,
                    pins: PINS,
                    config: config::Config,
                    clocks: rcc::Clocks,
                ) -> Result<Self, config::InvalidConfig>
                where
                    PINS: Pins<$USARTX>,
                {
                    use self::config::*;

                    // NOTE(safety) This executes only during initialisation
                    let rcc = unsafe { &(*RCC::ptr()) };

                    // Enable clock for USART
                    rcc.$apbXenr.modify(|_, w| w.$usartXen().set_bit());

                    let extended_divider = (clocks.$pclkX().0 << 4) / config.baudrate.0;
                    let mantissa = extended_divider >> 8;
                    let fraction = (extended_divider - (mantissa << 8)) >> 4;

                    // NOTE(safety) uses .bits for ease of writing a whole word.
                    // No reserved or read-only bits in this register
                    usart.brr.write(|w| unsafe { w.bits((mantissa << 4) | fraction) });

                    // Reset other registers to disable advanced USART features
                    usart.cr2.reset();
                    usart.cr3.reset();

                    // Enable transmission and receiving
                    // and configure frame
                    usart.cr1.write(|w| {
                        w.ue()
                            .set_bit()
                            .te()
                            .set_bit()
                            .re()
                            .set_bit()
                            .m()
                            .bit(match config.wordlength {
                                WordLength::DataBits8 => false,
                                WordLength::DataBits9 => true,
                            })
                            .pce()
                            .bit(match config.parity {
                                Parity::ParityNone => false,
                                _ => true,
                            })
                            .ps()
                            .bit(match config.parity {
                                Parity::ParityOdd => true,
                                _ => false,
                            })
                    });

                    Ok(Serial { usart, pins }.config_stop(config))
                }

                /// Starts listening for an interrupt event
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().set_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().set_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().set_bit())
                        },
                    }
                }

                /// Stop listening for an interrupt event
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().clear_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().clear_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().clear_bit())
                        },
                    }
                }

                /// Return true if the line idle status is set
                pub fn is_idle(& self) -> bool {
                    // NOTE(Safety) Atomic read on stateless register
                    unsafe { (*$USARTX::ptr()).sr.read().idle().bit_is_set() }
                }

                /// Return true if the tx register is empty (and can accept data)
                pub fn is_txe(& self) -> bool {
                    // NOTE(Safety) Atomic read on stateless register
                    unsafe { (*$USARTX::ptr()).sr.read().txe().bit_is_set() }
                }

                /// Return true if the rx register is not empty (and can be read)
                pub fn is_rxne(& self) -> bool {
                    // NOTE(Safety) Atomic read on stateless register
                    unsafe { (*$USARTX::ptr()).sr.read().rxne().bit_is_set() }
                }

                pub fn split(self) -> (Tx<$USARTX>, Rx<$USARTX>) {
                    (
                        Tx {
                            _usart: PhantomData,
                        },
                        Rx {
                            _usart: PhantomData,
                        },
                    )
                }
                pub fn release(self) -> ($USARTX, PINS) {
                    (self.usart, self.pins)
                }
            }

            impl<PINS> serial::Read for Serial<$USARTX, PINS> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let mut rx: Rx<$USARTX> = Rx {
                        _usart: PhantomData,
                    };
                    rx.read()
                }
            }

            impl serial::Read for Rx<$USARTX> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    // NOTE(Safety) Atomic read on stateless register
                    let sr = unsafe { (*$USARTX::ptr()).sr.read() };

                    // Any error requires the dr to be read to clear
                    if sr.pe().bit_is_set()
                        || sr.fe().bit_is_set()
                        || sr.nf().bit_is_set()
                        || sr.ore().bit_is_set()
                    {
                        // NOTE(Safety) Atomic read on stateless register
                        unsafe { (*$USARTX::ptr()).dr.read() };
                    }

                    Err(if sr.pe().bit_is_set() {
                        nb::Error::Other(Error::Parity)
                    } else if sr.fe().bit_is_set() {
                        nb::Error::Other(Error::Framing)
                    } else if sr.nf().bit_is_set() {
                        nb::Error::Other(Error::Noise)
                    } else if sr.ore().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.rxne().bit_is_set() {
                        // NOTE(read_volatile) see `write_volatile` below
                        return Ok(unsafe { ptr::read_volatile(&(*$USARTX::ptr()).dr as *const _ as *const u8) });
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl<PINS> serial::Write for Serial<$USARTX, PINS> {
                type Error = Error;

                fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
                    let mut tx: Tx<$USARTX> = Tx {
                        _usart: PhantomData,
                    };
                    tx.write_str(s)
                }

                fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
                    let mut tx: Tx<$USARTX> = Tx {
                        _usart: PhantomData,
                    };
                    tx.write_char(c)
                }
            }

            impl serial::Write for Tx<$USARTX> {
                type Error = Error;

                fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
                    for character in s.chars() {
                        self.write_char(character);
                    }
                    Ok(())
                }

                fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
                    // NOTE(Safety) atomic read with no side effects
                    let sr = unsafe { (*$USARTX::ptr()).sr.read() };
                    while !sr.txe().bit_is_set() {}
                    // NOTE(Safety) atomic write to stateless register
                    // NOTE(write_volatile) 8-bit write that's not possible through the svd2rust API
                    unsafe { ptr::write_volatile(&(*$USARTX::ptr()).dr as *const _ as *mut _, c as u8) }
                    Ok(())
                }
            }
        )+
    }
}

macro_rules! instances {
    ($(
        $USARTX:ident: ($usartX:ident, $apbXenr:ident, $usartXen:ident, $pclkX:ident),
    )+) => {
        $(
            impl<PINS> Serial<$USARTX, PINS> {
                fn config_stop(self, config: config::Config) -> Self {
                    use crate::stm32pac::usart1::cr2::STOP_A;
                    use self::config::*;

                    self.usart.cr2.write(|w| {
                        w.stop().variant(match config.stopbits {
                            StopBits::STOP0P5 => STOP_A::STOP0P5,
                            StopBits::STOP1 => STOP_A::STOP1,
                            StopBits::STOP1P5 => STOP_A::STOP1P5,
                            StopBits::STOP2 => STOP_A::STOP2,
                        })
                    });
                    self
                }
            }

        )+

        hal_usart_impl! {
            $( $USARTX: ($usartX, $apbXenr, $usartXen, $pclkX), )+
        }

        $(
            impl<PINS> UsartExt<PINS> for $USARTX
            where
                PINS: Pins<$USARTX>, {
                type Serial = Serial<$USARTX, PINS>;

                fn constrain(self,
                    pins: PINS,
                    config: config::Config,
                    clocks: rcc::Clocks,
                ) -> Result<Self::Serial, config::InvalidConfig> {
                    Serial::$usartX(self, pins, config, clocks)
                }
            }
        )+
    }
}

// Type definition macros. NOTE: This is not configuration! No
// need to remove these if unused, they exist only in the type
// system at this point.
instances! {
    USART1: (usart1, apb2enr, usart1en, pclk2),
    USART2: (usart2, apb1enr, usart2en, pclk1),
    USART3: (usart3, apb1enr, usart3en, pclk1),
    USART6: (usart6, apb2enr, usart6en, pclk2),
}
