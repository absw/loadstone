/// Quad SPI configured in Indirect mode.
///
/// Indirect mode forces all communication to occur through writes
/// and reads to QSPI registers.
pub trait Indirect {
    type Error;

    fn write(
        &mut self,
        instruction: Option<u8>,
        address: Option<u32>,
        data: Option<&[u8]>,
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error>;

    fn read(
        &mut self,
        instruction: Option<u8>,
        address: Option<u32>,
        data: &mut [u8],
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error>;
}
