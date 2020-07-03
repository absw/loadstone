/// Quad SPI configured in Indirect mode.
///
/// Indirect mode forces all communication to occur through writes
/// and reads to QSPI registers.
pub trait Indirect {
    type Error;
    type Instruction;
    type Address;

    fn write(
        &mut self,
        instruction: Option<Self::Instruction>,
        address: Option<Self::Address>,
        data: Option<&[u8]>,
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error>;

    fn read(
        &mut self,
        instruction: Option<Self::Instruction>,
        address: Option<Self::Address>,
        data: &mut [u8],
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error>;
}
