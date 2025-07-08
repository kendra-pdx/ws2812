pub trait ToBits {
    fn to_bits<Bit: From<bool>>(self) -> impl Iterator<Item = Bit>;
}

impl ToBits for u8 {
    fn to_bits<Bit: From<bool>>(self) -> impl Iterator<Item = Bit> {
        [
            (self & 0b1000_0000) != 0,
            (self & 0b0100_0000) != 0,
            (self & 0b0010_0000) != 0,
            (self & 0b0001_0000) != 0,
            (self & 0b0000_1000) != 0,
            (self & 0b0000_0100) != 0,
            (self & 0b0000_0010) != 0,
            (self & 0b0000_0001) != 0,
        ].into_iter().map(Bit::from)
    }
}