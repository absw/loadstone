use core::marker::PhantomData;
use stm32f4::stm32f429;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The type to split the GPIO into
    type GpioWrapper;

    /// Splits the GPIO block into independent pins and registers
    fn split(self, rcc: &mut stm32f429::RCC) -> Self::GpioWrapper;
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

/// Alternate function 0 (type state)
pub struct AF0;
/// Alternate function 1 (type state)
pub struct AF1;
/// Alternate function 2 (type state)
pub struct AF2;
/// Alternate function 3 (type state)
pub struct AF3;
/// Alternate function 4 (type state)
pub struct AF4;
/// Alternate function 5 (type state)
pub struct AF5;
/// Alternate function 6 (type state)
pub struct AF6;
/// Alternate function 7 (type state)
pub struct AF7;
/// Alternate function 8 (type state)
pub struct AF8;
/// Alternate function 9 (type state)
pub struct AF9;
/// Alternate function 10 (type state)
pub struct AF10;
/// Alternate function 11 (type state)
pub struct AF11;
/// Alternate function 12 (type state)
pub struct AF12;
/// Alternate function 13 (type state)
pub struct AF13;
/// Alternate function 14 (type state)
pub struct AF14;
/// Alternate function 15 (type state)
pub struct AF15;

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

macro_rules! gpio_inner {
    ($GPIOx:ident, $gpiox:ident, $enable_pin:ident, $reset_pin:ident, $Pxx:ident, [
        $($Pxi:ident: ($pxi:ident, $i:expr, $default_mode:ty), )+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use core::marker::PhantomData;
            use crate::hal::gpio::OutputPin;
            use stm32f4::stm32f429;

            // Lower case for identifier concatenation
            #[allow(unused_imports)]
            use stm32f4::stm32f429::{
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

                fn split(self, rcc: &mut stm32f429::RCC) -> GpioWrapper {
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
                pub struct $Pxi<MODE> {
                    _mode: PhantomData<MODE>,
                }

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

                impl $Pxi<AF7> {
                    #[allow(dead_code)]
                    fn new() -> Self {
                        let pin = $Pxi::<Input<Floating>> { _mode : PhantomData };
                        pin.into_af7()
                    }
                }

                impl<MODE> $Pxi<MODE> {
                    pub fn into_af7(
                        self,
                    ) -> $Pxi<AF7> {
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
