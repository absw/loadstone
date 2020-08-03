use crate::hal::serial;

pub struct MockSerial {}

impl serial::Write for MockSerial {
    type Error = ();
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl serial::Read for MockSerial {
    type Error = ();
    fn read(&mut self) -> nb::Result<u8, Self::Error> { Ok(0) }
}
