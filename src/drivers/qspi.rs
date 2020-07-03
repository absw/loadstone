use crate::{
    drivers::gpio::*,
    hal::qspi,
    pin_configuration::*,
    stm32pac::{QUADSPI as QuadSpiPeripheral, RCC},
};
use core::marker::PhantomData;
use nb::block;

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// Sealed trait for all QSPI capable pins.
pub unsafe trait ClkPin: private::Sealed {}
pub unsafe trait Bk1CsPin: private::Sealed {}
pub unsafe trait Bk2CsPin: private::Sealed {}
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
seal_pins!(Bk1CsPin: [Pb6<AF10>, Pg6<AF10>,]);
#[cfg(feature = "stm32f412")]
seal_pins!(Bk2CsPin: [Pc11<AF9>,]);
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

// Mode Typestates
pub mod mode {
    pub struct Single;
    pub struct Dual;
    pub struct Quad;
}

/// Whether bits are clocked on both edges
#[derive(PartialEq, Debug)]
pub enum DataRate {
    Single,
    /// Unimplemented
    Double,
}

/// Number of flash memories sharing a bus
#[derive(PartialEq, Debug)]
pub enum FlashMode {
    Single,
    /// Unimplemented
    Double,
}

/// QuadSPI configuration
pub struct Config<MODE> {
    data_rate: DataRate,
    flash_mode: FlashMode,
    flash_size_bits: u8,
    _marker: PhantomData<MODE>,
}

/// Marker trait for a tuple of pins that work for a given QSPI in Single mode
pub trait SingleModePins {}

impl<CLK, CS, IO0, IO1> SingleModePins for (CLK, CS, IO0, IO1)
where
    CLK: ClkPin,
    CS: Bk1CsPin,
    IO0: Bk1Io0Pin,
    IO1: Bk1Io1Pin,
{
}

/// QuadSPI abstraction
pub struct QuadSpi<PINS, MODE> {
    _pins: PINS,
    qspi: QuadSpiPeripheral,
    config: Config<MODE>,
    _marker: PhantomData<MODE>,
}

pub struct Instruction(u8);

pub enum Error {
    DummyCyclesValueOutOfRange,
    MisalignedData,
}

impl<MODE> Default for Config<MODE> {
    fn default() -> Self {
        Config {
            data_rate: DataRate::Single,
            flash_mode: FlashMode::Single,
            flash_size_bits: 24,
            _marker: PhantomData::default(),
        }
    }
}

impl<MODE> Config<MODE> {
    pub fn single(self) -> Config<mode::Single> {
        Config {
            data_rate: self.data_rate,
            flash_mode: self.flash_mode,
            flash_size_bits: self.flash_size_bits,
            _marker: PhantomData::default(),
        }
    }

    pub fn double(self) -> Config<mode::Dual> {
        Config {
            data_rate: self.data_rate,
            flash_mode: self.flash_mode,
            flash_size_bits: self.flash_size_bits,
            _marker: PhantomData::default(),
        }
    }

    pub fn quad(self) -> Config<mode::Quad> {
        Config {
            data_rate: self.data_rate,
            flash_mode: self.flash_mode,
            flash_size_bits: self.flash_size_bits,
            _marker: PhantomData::default(),
        }
    }

    pub fn with_data_rate(mut self, data_rate: DataRate) -> Self {
        self.data_rate = data_rate;
        self
    }

    pub fn with_flash_mode(mut self, flash_mode: FlashMode) -> Self {
        self.flash_mode = flash_mode;
        self
    }

    pub fn with_flash_size(mut self, bits: u8) -> nb::Result<Self, ConfigError> {
        match bits {
            8 | 16 | 24 | 32 => {
                self.flash_size_bits = bits;
                Ok(self)
            }
            _ => Err(nb::Error::Other(ConfigError::InvalidFlashSize)),
        }
    }
}

pub enum ConfigError {
    NotYetImplemented,
    InvalidFlashSize,
}

impl<PINS> QuadSpi<PINS, mode::Single>
where
    PINS: SingleModePins,
{
    pub fn from_config(
        qspi: QuadSpiPeripheral,
        pins: PINS,
        config: Config<mode::Single>,
    ) -> nb::Result<Self, ConfigError> {
        if config.data_rate != DataRate::Single || config.flash_mode != FlashMode::Single {
            return Err(nb::Error::Other(ConfigError::NotYetImplemented));
        }

        // NOTE(safety) This executes only during initialisation, and only
        // performs single-bit atomic writes related to the QSPI peripheral
        let rcc = unsafe { &(*RCC::ptr()) };
        rcc.ahb3enr.modify(|_, w| w.qspien().set_bit());

        // NOTE(safety) The unsafe "bits" method is used to write multiple bits conveniently.
        // Applies to all unsafe blocks in this function unless specified otherwise.
        // Prescaler bypass (AHB clock frequency)
        qspi.cr.modify(|_, w| unsafe { w.prescaler().bits(0) });

        // Fifo threshold 4 (fifo flag up when 4 bytes are free to write)
        qspi.cr.modify(|_, w| unsafe { w.fthres().bits(4u8) });

        let fsize = config.flash_size_bits.saturating_sub(1u8);
        qspi.dcr.modify(|_, w| unsafe { w.fsize().bits(fsize) });

        // Enable
        qspi.cr.modify(|_, w| w.en().set_bit());

        Ok(Self { _pins: pins, config, qspi, _marker: PhantomData::default() })
    }
}

struct Status {
    busy: bool,
    fifo_threshold: bool,
}

impl<PINS, MODE> QuadSpi<PINS, MODE> {
    fn status(&self) -> Status {
        let flags = self.qspi.sr.read();
        Status { busy: flags.busy().bit(), fifo_threshold: flags.ftf().bit() }
    }

    fn write_word(&mut self, word: &[u8]) -> nb::Result<(), Error> {
        if self.status().busy || !self.status().fifo_threshold {
            Err(nb::Error::WouldBlock)
        } else {
            // gather up to four bytes into a single big_endian u32
            let mut byte_array = [0u8; 4];
            word.iter().enumerate().for_each(|(i, b)| byte_array[i] = *b);
            let word = u32::from_be_bytes(byte_array);

            // NOTE(safety) The unsafe "bits" method is used to write multiple bits conveniently.
            self.qspi.dr.write(|w| unsafe { w.bits(word) });
            Ok(())
        }
    }

    fn read_word(&mut self, word: &mut [u8]) -> nb::Result<(), Error> {
        if self.status().busy || !self.status().fifo_threshold {
            Err(nb::Error::WouldBlock)
        } else {
            // Store read in a big endian array
            let bytes = self.qspi.dr.read().bits().to_be_bytes();
            bytes.iter().enumerate().for_each(|(i, b)| word[i] = *b);
            Ok(())
        }
    }
}

impl<PINS> qspi::Indirect for QuadSpi<PINS, mode::Single> {
    type Error = Error;

    fn write(
        &mut self,
        instruction: Option<u8>,
        address: Option<u32>,
        data: Option<&[u8]>,
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error> {
        if dummy_cycles > 31 {
            return Err(nb::Error::Other(Error::DummyCyclesValueOutOfRange));
        }

        // TODO: Consider allowing misaligned writes.
        match data {
            Some(data) if data.len() % 4 != 0 => {
                return Err(nb::Error::Other(Error::MisalignedData))
            }
            _ => (),
        }

        let adsize = match self.config.flash_size_bits {
            8 => 0b00,
            16 => 0b01,
            24 => 0b10,
            32 => 0b11,
            _ => panic!("Invalid flash size"),
        };

        if self.status().busy {
            // Early yield if busy
            return Err(nb::Error::WouldBlock);
        }

        // NOTE(safety) The unsafe "bits" method is used to write multiple bits conveniently.
        // Applies to all unsafe blocks in this function unless specified otherwise.
        // Sets Data Length Register, configuring the amount of bytes to write.
        self.qspi.dlr.write(|w| unsafe {
            w.bits(if let Some(data) = data { data.len() as u32 } else { 0 })
        });

        // Configure Communicaton Configuration Register.
        // This sets up all rules for this QSPI write.
        self.qspi.ccr.write(|w| unsafe {
            if let Some(instruction) = instruction {
                w.imode().bits(0b01).instruction().bits(instruction)
            } else {
                w
            }
            .fmode()
            .bits(0b00) // indirect write mode
            .adsize()
            .bits(adsize)
            .admode()
            .bits(if address.is_some() { 0b01 } else { 0b00 })
            .dmode()
            .bits(if data.is_some() { 0b01 } else { 0b00 })
            .dcyc()
            .bits(dummy_cycles)
        });

        // Sets Address to write to.
        if let Some(address) = address {
            self.qspi.ar.write(|w| unsafe { w.bits(address) })
        };

        // Write loop (checking FIFO threshold to ensure it is possible to write 4 bytes).
        if let Some(data) = data {
            for word in data.chunks(4) {
                block!(self.write_word(word))?;
            }
        }
        Ok(())
    }

    fn read(
        &mut self,
        instruction: Option<u8>,
        address: Option<u32>,
        data: &mut [u8],
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error> {
        if dummy_cycles > 31 {
            return Err(nb::Error::Other(Error::DummyCyclesValueOutOfRange));
        }

        // TODO: Consider allowing misaligned reads.
        if data.len() % 4 != 0 {
            return Err(nb::Error::Other(Error::MisalignedData));
        }

        let adsize = match self.config.flash_size_bits {
            8 => 0b00,
            16 => 0b01,
            24 => 0b10,
            32 => 0b11,
            _ => panic!("Invalid flash size"),
        };

        if self.status().busy {
            // Early yield if busy
            return Err(nb::Error::WouldBlock);
        }
        // NOTE(safety) The unsafe "bits" method is used to write multiple bits conveniently.
        // Applies to all unsafe blocks in this function unless specified otherwise.
        // Sets Data Length Register, configuring the amount of bytes to read.
        self.qspi.dlr.write(|w| unsafe { w.bits(data.len() as u32) });

        // Configure Communicaton Configuration Register.
        // This sets up all rules for this QSPI read.
        self.qspi.ccr.write(|w| unsafe {
            if let Some(instruction) = instruction {
                w.imode().bits(0b01).instruction().bits(instruction)
            } else {
                w
            }
            .fmode()
            .bits(0b01) // indirect read mode
            .adsize()
            .bits(adsize)
            .admode()
            .bits(if address.is_some() { 0b01 } else { 0b00 })
            .dmode()
            .bits(0b01)
            .dcyc()
            .bits(dummy_cycles)
        });

        // Sets Address to read from.
        if let Some(address) = address {
            self.qspi.ar.write(|w| unsafe { w.bits(address) })
        };

        // Read loop (checking FIFO threshold to ensure it is possible to read 4 bytes).
        for word in data.chunks_mut(4) {
            block!(self.read_word(word))?;
        }
        Ok(())
    }
}
