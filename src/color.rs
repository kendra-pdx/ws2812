use core::{ops::Mul, u8};

use num_traits::{Num, ToPrimitive};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct RGB<C: Num> {
    r: C,
    g: C,
    b: C,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct RGBW<C: Num> {
    r: C,
    g: C,
    b: C,
    w: C,
}

pub type RGB8 = RGB<u8>;
pub type RGBF = RGB<f32>;

pub type RGBW8 = RGBW<u8>;
pub type RGBWF = RGBW<f32>;

impl<C: Num> RGB<C> {
    pub const fn new(r: C, g: C, b: C) -> Self {
        Self { r, g, b }
    }
}

impl<C: Num> RGBW<C> {
    pub const fn new(r: C, g: C, b: C, w: C) -> Self {
        Self { r, g, b, w }
    }
}

impl Mul<f32> for RGBW8 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        assert!(rhs >= 0.0 && rhs <= 1.0);
        let RGBW8 { r, g, b, w } = self;

        let scale = move |v: u8| {
            const MAX_U8_F: f32 = u8::MAX as f32;
            let v = v as f32;
            let v: f32 = (v / MAX_U8_F * rhs) * MAX_U8_F;
            v as u8
        };

        RGBW8 {
            r: scale(r),
            g: scale(g),
            b: scale(b),
            w: scale(w),
        }
    }
}

impl Mul<f32> for RGBWF {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        assert!(rhs >= 0.0 && rhs <= 1.0);
        let RGBWF { r, g, b, w } = self;
        let scale = move |v: f32| v * rhs;

        RGBWF {
            r: scale(r),
            g: scale(g),
            b: scale(b),
            w: scale(w),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Channel {
    R,
    G,
    B,
    W,
}

pub type ChannelOrder<const N: usize> = [Channel; N];
pub const GRB: ChannelOrder<3> = [Channel::G, Channel::R, Channel::B];
pub const RGB: ChannelOrder<3> = [Channel::R, Channel::G, Channel::B];
pub const BGR: ChannelOrder<3> = [Channel::B, Channel::G, Channel::R];

pub const RGBW: ChannelOrder<4> = [Channel::R, Channel::G, Channel::B, Channel::W];
pub const GRBW: ChannelOrder<4> = [Channel::G, Channel::R, Channel::B, Channel::W];
pub const BGRW: ChannelOrder<4> = [Channel::B, Channel::G, Channel::R, Channel::W];

pub trait ColorChannels<C: Num, const N: usize> {
    fn channels(self, order: ChannelOrder<N>) -> [C; N];
}

impl<RGB: Into<RGB8>> ColorChannels<u8, 3> for RGB {
    fn channels(self, order: ChannelOrder<3>) -> [u8; 3] {
        let mut result = [0; 3];
        let RGB8 { r, g, b } = self.into();
        for (i, &channel) in order.iter().enumerate() {
            match channel {
                Channel::R => result[i] = r,
                Channel::G => result[i] = g,
                Channel::B => result[i] = b,
                Channel::W => result[i] = 0, // White channel is not supported in RGB8
            }
        }
        result
    }
}

impl<RGBW: Into<RGBW8>> ColorChannels<u8, 4> for RGBW {
    fn channels(self, order: ChannelOrder<4>) -> [u8; 4] {
        let mut result = [0; 4];
        let RGBW8 { r, g, b, w } = self.into();
        for (i, &channel) in order.iter().enumerate() {
            match channel {
                Channel::R => result[i] = r,
                Channel::G => result[i] = g,
                Channel::B => result[i] = b,
                Channel::W => result[i] = w,
            }
        }
        result
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

impl Into<RGBW8> for RGBWF {
    fn into(self) -> RGBW8 {
        let r = (self.r * 255.).to_u8().unwrap_or(0);
        let g = (self.g * 255.).to_u8().unwrap_or(0);
        let b = (self.b * 255.).to_u8().unwrap_or(0);
        let w = (self.w * 255.).to_u8().unwrap_or(0);

        RGBW8::new(r, g, b, w)
    }
}

#[cfg(test)]
mod tests {
    use crate::bits::*;
    use crate::color::*;

    #[test]
    fn test_color_conversion() {
        const WHITE_8: RGB<u8> = RGB::new(0xff, 0xff, 0xff);
        const WHITE_F: RGB<f32> = RGB::new(1., 1., 1.);
        assert_eq!(WHITE_8, WHITE_F.into());

        // let bits: [u8; 24] = WHITE_8.to_bits(u8::from);
    }

    #[test]
    fn test_channels() {
        let rgb = RGB8::new(255, 0, 0);
        let channels = rgb.channels(GRB);
        assert_eq!(channels, [0, 255, 0]);
    }
}
