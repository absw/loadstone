use crate::error::Error;

#[derive(Debug, Copy, Clone)]
pub struct FakeError;

impl From<FakeError> for Error {
    fn from(_error: FakeError) -> Self {
        Error::DeviceError("A fake error occurred [TESTING ONLY]")
    }
}
