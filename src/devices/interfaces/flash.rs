/// Abstract Flash read, generic over any concept of memory "area"
pub trait Read<A> {
    type Data;
    type Error;
    fn read(area: A) -> nb::Result<Self::Data, Self::Error>;
}

/// Abstract Flash write, generic over any concept of memory "area"
pub trait Write<A> {
    type Data;
    type Error;
    fn write(area: A, data: Self::Data) -> nb::Result<(), Self::Error>;
}

/// Abstract Flash erase, generic over any concept of memory "area"
pub trait Erase<A> {
    type Error;
    fn erase(area: A) -> nb::Result<(), Self::Error>;
}
