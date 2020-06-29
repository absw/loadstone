/// Abstract Flash read, generic over any memory "area"
pub trait Read<A> {
    type Data;
    type Error;
    fn read(&mut self, area: A) -> nb::Result<Self::Data, Self::Error>;
}

/// Abstract Flash write, generic over any memory "area"
pub trait Write<A> {
    type Data;
    type Error;
    fn write(&mut self, area: A, data: Self::Data) -> nb::Result<(), Self::Error>;
}

/// Abstract Flash erase, generic over any memory "area"
pub trait Erase<A> {
    type Error;
    fn erase(&mut self, area: A) -> nb::Result<(), Self::Error>;
}

/// Abstract mass erase
pub trait BulkErase {
    type Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error>;
}
