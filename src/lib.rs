#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

use cfg_if::cfg_if;
use core::time::Duration;

use crate::color::ColorChannels;

mod bits;
pub mod color;

#[cfg(feature = "esp")]
pub mod esp;

cfg_if! {
    if #[cfg(feature = "timings_spec")] {
        // https://cdn-shop.adafruit.com/datasheets/WS2812.pdf
        pub const T0H: Duration = Duration::from_nanos(350);
        pub const T0L: Duration = Duration::from_nanos(800);

        pub const T1H: Duration = Duration::from_nanos(700);
        pub const T1L: Duration = Duration::from_nanos(600);
    } else {
        // https://learn.adafruit.com/adafruit-neopixel-uberguide/advanced-coding
        pub const T0H: Duration = Duration::from_nanos(400);
        pub const T0L: Duration = Duration::from_nanos(850);

        pub const T1H: Duration = Duration::from_nanos(800);
        pub const T1L: Duration = Duration::from_nanos(450);
    }
}

pub struct Symbol {
    high: Duration,
    low: Duration,
}

impl Symbol {
    pub const T1: Symbol = Symbol {
        high: T1H,
        low: T1L,
    };

    pub const T0: Symbol = Symbol {
        high: T0H,
        low: T0L,
    };
}

impl From<bool> for Symbol {
    fn from(value: bool) -> Self {
        if value { Symbol::T1 } else { Symbol::T0 }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("WS2812 Error: {msg}")]
pub struct WS2812Error {
    msg: alloc::string::String,
}

pub trait WS2812<const N_COLOR_CHANNELS: usize>
where
    Self: Sized,
{
    fn write<Px: ColorChannels<u8, N_COLOR_CHANNELS>>(self, pixels: impl Iterator<Item = Px>) -> Result<Self, WS2812Error>;
}

#[cfg(feature = "defmt")]
impl defmt::Format for WS2812Error {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "WS2812 Error: {}", self.msg.as_str());
    }
}
