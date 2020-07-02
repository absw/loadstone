
/// Quad SPI configured in Indirect mode.
///
/// Indirect mode forces all communication to occur through writes
/// and reads to QSPI registers.
pub trait Indirect {
    type Error;
    type Instruction;
    type Address;
    type Word;

    fn write(instruction: Self::Instruction, address: Option<Self::Address>, data: &[Self::Word]) -> nb::Result<(), Self::Error>;
    fn read(instruction: Self::Instruction, address: Option<Self::Address>, data: &mut [Self::Word]) -> nb::Result<(), Self::Error>;
}
