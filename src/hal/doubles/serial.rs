use crate::hal::serial;

pub struct MockSerial {}

impl serial::Write for MockSerial {
    type Error = ();
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> { Ok(()) }
}

impl serial::Read for MockSerial {
    type Error = ();
    fn read(&mut self) -> nb::Result<u8, Self::Error> { Ok(0) }
}
