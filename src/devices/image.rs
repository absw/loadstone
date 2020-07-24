use crate::utilities::memory::Address;

pub struct GlobalHeader {
    test_buffer: [u8; 4],
}

pub struct ImageHeader {
    size: usize,
}

pub struct Bank<A: Address> {
    pub size: usize,
    pub location: A,
}
