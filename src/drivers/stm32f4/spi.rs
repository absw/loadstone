use crate::{
    drivers::stm32f4::gpio::*,
    hal::spi::FullDuplex,
    ports::pin_configuration::*,
    stm32pac::{RCC, SPI1},
};
use core::{marker::PhantomData, mem::size_of};

const BAUD_RATE_DIVIDER: u8 = 4;
pub type SpiAf = AF5;

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// Sealed trait for all SPI capable pins.
pub unsafe trait MisoPin<SPI>: private::Sealed {}
pub unsafe trait MosiPin<SPI>: private::Sealed {}
pub unsafe trait SckPin<SPI>: private::Sealed {}
pub unsafe trait NssPin<SPI>: private::Sealed {}

#[allow(unused)]
macro_rules! seal_pins { ($function:ty: [$($pin:ty,)+]) => {
    $(
        unsafe impl $function for $pin {}
        impl private::Sealed for $pin {}
    )+
};}

#[cfg(feature = "stm32f412")]
seal_pins!(NssPin<SPI1>: [Pa4<SpiAf>, Pa15<SpiAf>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(SckPin<SPI1>: [Pa5<SpiAf>, Pb3<SpiAf>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(MisoPin<SPI1>: [Pa6<SpiAf>, Pb4<SpiAf>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(MosiPin<SPI1>: [Pa7<SpiAf>, Pb5<SpiAf>,]);

/// Marker trait for a tuple of pins that work for a given SPI.
pub trait Pins<SPI> {}

impl<SPI, MISO, MOSI, SCK> Pins<SPI> for (MISO, MOSI, SCK)
where
    MISO: MisoPin<SPI>,
    MOSI: MosiPin<SPI>,
    SCK: SckPin<SPI>,
{
}

/// SPI abstraction
pub struct Spi<SPI, PINS, WORD> {
    spi: SPI,
    _pins: PINS,
    _word: PhantomData<WORD>,
    awaiting_receive: bool,
}

#[derive(Debug)]
pub enum FullDuplexSpiError {
    OutOfOrderOperation,
}

pub enum Mode {
    Zero,
    One,
    Two,
    Three,
}

#[allow(unused_macros)]
macro_rules! hal_spi_impl {
    ($(
        $SPIX:ident: ($word: tt, $spiX:ident, $apbXenr:ident, $spiXen:ident,  $pclkX:ident)
    )+) => {
        $(
            impl<PINS> Spi<$SPIX, PINS, $word> {
                pub fn $spiX(
                    spi: $SPIX, pins: PINS, mode: Mode
                ) -> Self
                    where PINS: Pins<$SPIX>,
                {
                    // NOTE(safety) This executes only during initialisation.
                    let rcc = unsafe { &(*RCC::ptr()) };

                    // Enable clock for SPI
                    rcc.$apbXenr.modify(|_, w| w.$spiXen().set_bit());

                    // Baud rate divider
                    spi.cr1.modify(|_, w| w.br().bits(BAUD_RATE_DIVIDER));

                    // Mode bits
                    match mode {
                        Mode::Zero => spi.cr1.modify(|_, w| w.cpol().clear_bit().cpha().clear_bit()),
                        Mode::One => spi.cr1.modify(|_, w| w.cpol().clear_bit().cpha().set_bit()),
                        Mode::Two => spi.cr1.modify(|_, w| w.cpol().set_bit().cpha().clear_bit()),
                        Mode::Three => spi.cr1.modify(|_, w| w.cpol().set_bit().cpha().set_bit()),
                    }

                    // Software slave management
                    spi.cr1.modify(|_, w| w.ssm().set_bit());

                    // Word length
                    match size_of::<$word>() {
                        1 => spi.cr1.modify(|_, w| w.dff().clear_bit()),
                        2 => spi.cr1.modify(|_, w| w.dff().set_bit()),
                        _ => panic!("Unsupported word size"),
                    }

                    // Master mode and enable
                    spi.cr1.modify(|_, w| w.mstr().set_bit().spe().set_bit());

                    Self { spi, _pins: pins, _word: PhantomData, awaiting_receive: false }
                }

                pub fn is_ready_to_transmit(&self) -> bool {
                    self.spi.sr.read().txe().bit_is_set() && !self.awaiting_receive
                }

                pub fn is_ready_to_receive(&self) -> bool {
                    self.spi.sr.read().rxne().bit_is_set() && self.awaiting_receive
                }

                pub fn is_busy(&self) -> bool {
                    self.spi.sr.read().bsy().bit_is_set()
                }
            }

            impl<PINS> FullDuplex<$word> for Spi<$SPIX, PINS, $word> {
                type Error = FullDuplexSpiError;

                fn transmit(&mut self, word: Option<$word>) -> nb::Result<(), Self::Error> {
                    if self.awaiting_receive {
                        return Err(nb::Error::Other(FullDuplexSpiError::OutOfOrderOperation))
                    }

                    if !self.is_ready_to_transmit() || self.is_busy() {
                        return Err(nb::Error::WouldBlock);
                    }

                    let word = word.unwrap_or(0) as u16;
                    self.spi.dr.write(|w| w.dr().bits(word));
                    self.awaiting_receive = true;
                    Ok(())
                }

                fn receive(&mut self) -> nb::Result<$word, Self::Error> {
                    if !self.awaiting_receive {
                        return Err(nb::Error::Other(FullDuplexSpiError::OutOfOrderOperation))
                    }

                    if !self.is_ready_to_receive() || self.is_busy() {
                        return Err(nb::Error::WouldBlock);
                    }

                    self.awaiting_receive = false;
                    Ok(self.spi.dr.read().dr().bits() as $word)
                }
            }
        )+
    }
}

hal_spi_impl!(
    SPI1: (u8, spi1, apb2enr, spi1en, pclk2)
    SPI1: (u16, spi1, apb2enr, spi1en, pclk2)
);
