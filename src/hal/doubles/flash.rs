use super::error::FakeError;
use crate::hal::flash;
use std::{
    cmp::max,
    ops::{Add, Sub},
};

pub struct FakeFlash {
    base: Address,
    length: usize,
    data: Vec<u8>,
}

impl FakeFlash {
    pub fn new(base: Address) -> FakeFlash { FakeFlash { base, data: Vec::new(), length: MB!(16) } }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub struct Address(pub u32);

impl flash::ReadWrite for FakeFlash {
    type Error = FakeError;
    type Address = Address;
    fn read(&mut self, address: Self::Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        if address < self.base {
            Err(nb::Error::Other(FakeError))
        } else {
            self.data.iter().skip(address - self.base).zip(bytes).for_each(|(i, o)| *o = *i);
            Ok(())
        }
    }
    fn write(&mut self, address: Self::Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        if address < self.base {
            Err(nb::Error::Other(FakeError))
        } else {
            let offset = address - self.base;
            self.data.resize_with(max(self.data.len(), offset + bytes.len()), Default::default);
            self.data.iter_mut().skip(address - self.base).zip(bytes).for_each(|(o, i)| *o = *i);
            Ok(())
        }
    }
    fn range(&self) -> (Self::Address, Self::Address) { (self.base, self.base + self.length) }
    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        self.data.clear();
        Ok(())
    }
}

impl Add<usize> for Address {
    type Output = Address;
    fn add(self, rhs: usize) -> Self::Output { Address(self.0 + rhs as u32) }
}

impl Sub<usize> for Address {
    type Output = Address;
    fn sub(self, rhs: usize) -> Self::Output { Address(self.0.saturating_sub(rhs as u32)) }
}

impl Sub<Address> for Address {
    type Output = usize;
    fn sub(self, rhs: Address) -> Self::Output { self.0.saturating_sub(rhs.0) as usize }
}

impl From<Address> for usize {
    fn from(address: Address) -> Self { address.0 as usize }
}
