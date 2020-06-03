//! This GPIO implementation is based on [typestates](https://rust-embedded.github.io/book/static-guarantees/typestate-programming.html).
//!
//! What this means is that pin configuration is encoded in the type
//! system, making it statically impossible to misuse a pin (e.g. there's
//! no "write" operation on a pin that has been configured as input).
use core::marker::PhantomData;
use crate::stm32pac;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The type to split the GPIO into
    type GpioWrapper;

    /// Splits the GPIO block into independent pins and registers
    fn split(self, rcc: &mut stm32pac::RCC) -> Self::GpioWrapper;
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}
/// Floating input (type state)
pub struct Floating;
/// Pulled down input (type state)
pub struct PullDown;
/// Pulled up input (type state)
pub struct PullUp;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}
/// Push pull output (type state)
pub struct PushPull;
/// Open drain output (type state)
pub struct OpenDrain;

#[macro_export]
macro_rules! alternate_functions {
    ($($i:expr, )+) => { $( paste::item! {
        /// Alternate function (type state)
        pub struct [<AF $i>];
    } )+ }
}
alternate_functions!(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,);

#[macro_export]
macro_rules! pin_rows {
    ($($x:ident,)+) => {
        use core::marker::PhantomData;
        $(
            pin_row!($x, [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,]);
        )+
    }
}
macro_rules! pin_row {
    ($x:ident, [$($i:expr,)+]) => { $( paste::item! {
        /// Pin with a MODE typestate
        pub struct [<P $x $i>]<MODE> {
            _mode: PhantomData<MODE>,
        }
    } )+
    }
}

/// Instantiates a gpio pin row with default modes per available pin
#[macro_export]
macro_rules! gpio {
    ($x: ident, [
        $( ($i:expr, $default_mode:ty), )+
    ]) => {

        // Macro black magic. the "paste" crate generates a context where anything bounded by "[<"
        // and ">]" delimiters gets concatenated in a single identifier post macro expansion. For
        // example, "[<GPIO $x>]" becomes "GPIOa" when "$x" represents "a". This is used to
        // expand the outer level, simplified "gpio!" instantiation macro into the complex one.
        paste::item_with_macros! {
            gpio_inner!([<GPIO $x>], [<gpio $x>], [<gpio $x en>], [<gpio $x rst>], [<P $x x>], [
                $( [<P $x $i>]: ([<p $x $i>], $i, $default_mode), )+
            ]);
        }
    }
}

macro_rules! into_af {
    ($GPIOx:ident, $i:expr, $Pxi:ident, $pxi:ident, [$($af_i:expr, )+]) => { $( paste::item! {
        pub fn [<into_af $af_i>](self) -> $Pxi<[<AF $af_i>]> {
            let offset = 2 * $i;

            // alternate function mode
            let mode = 0b10;

            // NOTE(safety) atomic read-modify-write operation to a stateless register.
            // It is also safe because pins are only reachable by splitting a GPIO struct,
            // which preserves single ownership of each pin.
            unsafe {
                (*$GPIOx::ptr()).moder.modify(|r, w|
                    w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                );
            }

            let af = 7;
            let offset = 4 * ($i % 8);

            if $i < 8 {
                // NOTE(safety) atomic read-modify-write operation to a stateless register.
                // It is also safe because pins are only reachable by splitting a GPIO struct,
                // which preserves single ownership of each pin.
                unsafe {
                    (*$GPIOx::ptr()).afrl.modify(|r, w|
                        w.bits((r.bits() & !(0b1111 << offset)) | (af << offset))
                    );
                }
            } else {
                // NOTE(safety) atomic read-modify-write operation to a stateless register.
                // It is also safe because pins are only reachable by splitting a GPIO struct,
                // which preserves single ownership of each pin.
                unsafe {
                    (*$GPIOx::ptr()).afrh.modify(|r, w|
                        w.bits((r.bits() & !(0b1111 << offset)) | (af << offset))
                    );
                }
            }

            $Pxi { _mode: PhantomData }
        }
} )+ }
}

macro_rules! new_af {
    ($GPIOx:ident, $i:expr, $Pxi:ident, $pxi:ident, [$($af_i:expr, )+]) => { $( paste::item! {
        impl $Pxi<[<AF $af_i>]> {
            #[allow(dead_code)]
            fn new() -> Self {
                let pin = $Pxi::<Input<Floating>> { _mode : PhantomData };
                pin.[<into_af $af_i>]()
            }
        }
} )+ }
}

macro_rules! gpio_inner {
    ($GPIOx:ident, $gpiox:ident, $enable_pin:ident, $reset_pin:ident, $Pxx:ident, [
        $($Pxi:ident: ($pxi:ident, $i:expr, $default_mode:ty), )+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use core::marker::PhantomData;
            use crate::hal::gpio::OutputPin;
            use crate::pin_configuration::*;

            // Lower case for identifier concatenation
            #[allow(unused_imports)]
            use crate::stm32pac::{
                GPIOA as GPIOa,
                GPIOB as GPIOb,
                GPIOC as GPIOc,
                GPIOD as GPIOd,
                GPIOE as GPIOe,
                GPIOF as GPIOf,
                GPIOG as GPIOg,
                GPIOH as GPIOh,
                GPIOI as GPIOi,
                GPIOJ as GPIOj,
                GPIOK as GPIOk,
            };

            use crate::drivers::gpio::*;

            /// GPIO parts
            pub struct GpioWrapper {
                $(
                    /// Pin
                    pub $pxi: $Pxi<$default_mode>,
                )+
            }

            impl GpioExt for $GPIOx {
                type GpioWrapper = GpioWrapper;

                fn split(self, rcc: &mut crate::stm32pac::RCC) -> GpioWrapper {
                    rcc.ahb1enr.modify(|_, w| w.$enable_pin().enabled());
                    rcc.ahb1rstr.modify(|_, w| w.$reset_pin().set_bit());
                    rcc.ahb1rstr.modify(|_, w| w.$reset_pin().clear_bit());

                    $(
                        let $pxi = $Pxi::<$default_mode>::new();
                    )+

                    GpioWrapper {
                        $($pxi,)+
                    }
                }
            }
            /// Partially erased pin
            pub struct $Pxx<MODE> {
                i: u8,
                _mode: PhantomData<MODE>,
            }

            impl<MODE> OutputPin for $Pxx<Output<MODE>> {
                fn set_high(&mut self) {
                    // NOTE(safety) atomic write to a stateless register. It is also safe
                    // because pins are only reachable by splitting a GPIO struct,
                    // which preserves single ownership of each pin.
                    unsafe { (*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << self.i)) }
                }

                fn set_low(&mut self) {
                    // NOTE(safety) atomic write to a stateless register. It is also safe
                    // because pins are only reachable by splitting a GPIO struct,
                    // which preserves single ownership of each pin.
                    unsafe { (*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << (16 + self.i))) }
                }
            }

            $(
                /// Pin
                impl $Pxi<Input<Floating>> {
                    #[allow(dead_code)]
                    fn new() -> Self {
                        $Pxi { _mode: PhantomData }
                    }
                }

                impl $Pxi<Output<PushPull>> {
                    #[allow(dead_code)]
                    fn new() -> Self {
                        let pin = $Pxi::<Input<Floating>> { _mode : PhantomData };
                        pin.into_push_pull_output()
                    }
                }

                impl $Pxi<Input<PullDown>> {
                    #[allow(dead_code)]
                    fn new() -> Self {
                        let pin = $Pxi::<Input<Floating>> { _mode : PhantomData };
                        pin.into_pull_down_input()
                    }
                }

                impl $Pxi<Input<PullUp>> {
                    #[allow(dead_code)]
                    fn new() -> Self {
                        let pin = $Pxi::<Input<Floating>> { _mode : PhantomData };
                        pin.into_pull_up_input()
                    }
                }

                impl $Pxi<Output<OpenDrain>> {
                    #[allow(dead_code)]
                    fn new() -> Self {
                        let pin = $Pxi::<Input<Floating>> { _mode : PhantomData };
                        pin.into_open_drain_output()
                    }
                }

                new_af!($GPIOx, $i, $Pxi, $pxi, [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,]);

                impl<MODE> $Pxi<MODE> {
                    into_af!($GPIOx, $i, $Pxi, $pxi, [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,]);

                    /// Configures the pin to operate as a floating input pin
                    pub fn into_floating_input(
                        self,
                    ) -> $Pxi<Input<Floating>> {
                        let offset = 2 * $i;

                        // input mode
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).moder.modify(|r, w| w.bits(r.bits() & !(0b11 << offset)) ); }

                        // no pull-up or pull-down
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).pupdr.modify(|r, w|  w.bits(r.bits() & !(0b11 << offset)) ); }

                        $Pxi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled down input pin
                    pub fn into_pull_down_input(
                        self,
                    ) -> $Pxi<Input<PullDown>> {
                        let offset = 2 * $i;

                        // input mode
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).moder.modify(|r, w| w.bits(r.bits() & !(0b11 << offset)) ); }

                        // pull-down
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).pupdr.modify(|r, w|
                            w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset))
                        ); }

                        $Pxi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled up input pin
                    pub fn into_pull_up_input(
                        self,
                    ) -> $Pxi<Input<PullUp>> {
                        let offset = 2 * $i;

                        // input mode
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).moder.modify(|r, w| w.bits(r.bits() & !(0b11 << offset)) ); }

                        // pull-up
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).pupdr.modify(|r, w|
                            w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset))
                        ); }

                        $Pxi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an open drain output pin
                    pub fn into_open_drain_output(
                        self,
                    ) -> $Pxi<Output<OpenDrain>> {
                        let offset = 2 * $i;

                        // general purpose output mode
                        let mode = 0b01;
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).moder.modify(|r, w|
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        ); }

                        // open drain output
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).otyper.modify(|r, w| w.bits(r.bits() | (0b1 << $i)) ); }

                        $Pxi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an push pull output pin
                    pub fn into_push_pull_output(
                        self,
                    ) -> $Pxi<Output<PushPull>> {
                        let offset = 2 * $i;

                        // general purpose output mode
                        let mode = 0b01;

                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).moder.modify(|r, w|
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        ); }

                        // push pull output
                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).otyper.modify(|r, w| w.bits(r.bits() & !(0b1 << $i)) ); }

                        $Pxi { _mode: PhantomData }
                    }
                }

                impl $Pxi<Output<OpenDrain>> {
                    /// Enables / disables the internal pull up
                    pub fn internal_pull_up(&mut self, on: bool) {
                        let offset = 2 * $i;

                        // NOTE(safety) atomic read-modify-write operation to a stateless register.
                        // It is also safe because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).pupdr.modify(|r, w|
                            w.bits(
                                (r.bits() & !(0b11 << offset)) | if on {
                                    0b01 << offset
                                } else {
                                    0
                                },
                            )
                        ); }
                    }
                }

                impl<MODE> $Pxi<Output<MODE>> {
                    /// Erases the pin number from the type
                    ///
                    /// This is useful when you want to collect the pins into an array where you
                    /// need all the elements to have the same type
                    pub fn downgrade(self) -> $Pxx<Output<MODE>> {
                        $Pxx {
                            i: $i,
                            _mode: self._mode,
                        }
                    }
                }

                impl<MODE> OutputPin for $Pxi<Output<MODE>> {
                    fn set_high(&mut self) {
                        // NOTE(safety) atomic write to a stateless register. It is also safe
                        // because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << $i)) }
                    }

                    fn set_low(&mut self) {
                        // NOTE(safety) atomic write to a stateless register. It is also safe
                        // because pins are only reachable by splitting a GPIO struct,
                        // which preserves single ownership of each pin.
                        unsafe { (*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << (16 + $i))) }
                    }
                }
            )+
        }
    }
}
