use crate::pin_configuration::*;
use crate::drivers::gpio::*;

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// Sealed trait for all QSPI capable pins.
pub unsafe trait ClkPin: private::Sealed {}
pub unsafe trait Bk1Io0Pin: private::Sealed {}
pub unsafe trait Bk1Io1Pin: private::Sealed {}
pub unsafe trait Bk1Io2Pin: private::Sealed {}
pub unsafe trait Bk1Io3Pin: private::Sealed {}
pub unsafe trait Bk2Io0Pin: private::Sealed {}
pub unsafe trait Bk2Io1Pin: private::Sealed {}
pub unsafe trait Bk2Io2Pin: private::Sealed {}
pub unsafe trait Bk2Io3Pin: private::Sealed {}

#[allow(unused)]
macro_rules! seal_pins { ($function:ty: [$($pin:ty,)+]) => {
    $(
        unsafe impl $function for $pin {}
        impl private::Sealed for $pin {}
    )+
};}

// There is no consistent alternate function for QSPI (varies between
// 9 and 10) so there is no type alias for QSPI AF.

#[cfg(feature = "stm32f412")]
seal_pins!(ClkPin: [Pb1<AF9>, Pb2<AF9>, Pd3<AF9>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk1Io0Pin: [Pc9<AF9>, Pd11<AF9>, Pf8<AF10>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk1Io1Pin: [Pc10<AF9>, Pd12<AF9>, Pf9<AF10>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk1Io2Pin: [Pc8<AF9>, Pe2<AF9>, Pf7<AF9>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk1Io3Pin: [Pa1<AF10>, Pd13<AF10>, Pf6<AF9>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk2Io0Pin: [Pa6<AF10>, Pe7<AF10>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk2Io1Pin: [Pa7<AF10>, Pe8<AF10>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk2Io2Pin: [Pc4<AF10>, Pe9<AF10>, Pg9<AF9>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk2Io3Pin: [Pc5<AF10>, Pe10<AF10>, Pg14<AF9>,]);
