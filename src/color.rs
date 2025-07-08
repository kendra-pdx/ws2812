use crate::bits::ToBits;
use num_traits::{Num, ToPrimitive};
use zerocopy::{FromBytes, IntoBytes};

#[derive(Debug, FromBytes, IntoBytes, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct RGB<C: Num> {
    r: C,
    g: C,
    b: C,
}

pub type RGB8 = RGB<u8>;
pub type RGBF = RGB<f32>;

impl<C: Num> RGB<C> {
    pub const fn new(r: C, g: C, b: C) -> Self {
        Self { r, g, b }
    }
}

impl Into<RGB8> for RGBF {
    fn into(self) -> RGB8 {
        let r = (self.r * 255.).to_u8().unwrap_or(0);
        let g = (self.g * 255.).to_u8().unwrap_or(0);
        let b = (self.b * 255.).to_u8().unwrap_or(0);
        RGB8::new(r, g, b)
    }
}

impl ToBits for RGB8 {
    fn to_bits<Bit: From<bool>>(self) -> impl Iterator<Item = Bit> {
        let g = self.g.to_bits();
        let r = self.r.to_bits();
        let b = self.b.to_bits();

        //‼️ important ‼️: GRB ORDER!
        [g, r, b].into_iter().flatten()
    }
}

#[cfg(test)]
mod tests {
    use crate::bits::*;
    use crate::color::RGB;

    #[test]
    fn test_color_conversion() {
        const WHITE_8: RGB<u8> = RGB::new(0xff, 0xff, 0xff);
        const WHITE_F: RGB<f32> = RGB::new(1., 1., 1.);
        assert_eq!(WHITE_8, WHITE_F.into());

        // let bits: [u8; 24] = WHITE_8.to_bits(u8::from);
    }
}
