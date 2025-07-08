use core::time::Duration;

use esp_hal::{gpio::Level, time::Rate};
use esp_hal::rmt::*;

use crate::{bits::*, color::RGB8, Symbol, WS2812Error, WS2812};
use alloc::vec::Vec;

pub struct EspWS2812<Tx: TxChannel> {
    tx: Tx,
    p1: u32,
    p0: u32,
}

impl<Tx: TxChannel> EspWS2812<Tx> {
    pub fn new(tx: Tx, clk_freq: Rate) -> Self {
        EspWS2812 { 
            tx, 
            p1: symbol_to_pulse_code(&clk_freq, &Symbol::T1),
            p0: symbol_to_pulse_code(&clk_freq, &Symbol::T0),
        }
    }
}

// https://docs.esp-rs.org/esp-idf-hal/src/esp_idf_hal/rmt.rs.html#137-144
pub fn duration_to_ticks(frequency: &Rate, duration: &Duration) -> u16 {
    const NANOS_PER_SECOND: u32 = 1_000_000_000;

    let ticks_hz = frequency.as_hz();
    let duration_ns = u32::try_from(duration.as_nanos())
        .expect("Overflow in duration_to_ticks");

    let ticks = duration_ns
        .checked_mul(ticks_hz)
        .expect("Overflow in duration_to_ticks")
        / NANOS_PER_SECOND;

    u16::try_from(ticks).expect("Overflow in duration_to_ticks")
}

fn symbol_to_pulse_code(clk_rate: &Rate, symbol: &Symbol) -> u32 {
    let Symbol { high, low } = symbol;

    let high_ticks = duration_to_ticks(clk_rate, high);
    let low_ticks = duration_to_ticks(clk_rate, low);

    PulseCode::new(Level::High, high_ticks, Level::Low, low_ticks)
}

impl<Tx: TxChannel> WS2812 for EspWS2812<Tx> {
    fn write(self, pixels: impl Iterator<Item = impl Into<RGB8>>) -> Result<Self, WS2812Error> {
        use core::iter;

        let end: u32 = PulseCode::empty();
        let data = pixels
            .map(|px| px.into())
            .flat_map(|px: RGB8| px.to_bits::<bool>())
            .map(|s| if s { self.p1 } else { self.p0 })
            .chain(iter::once(end))
            .collect::<Vec<_>>();

        let tx = self.tx.transmit(data.as_slice()).map_err(|e| e.into())?;

        let tx: Tx = tx.wait().map_err(|(e, _)| {
            e.into()
        })?;

        Ok(EspWS2812 { tx, ..self })
    }
}

impl Into<WS2812Error> for esp_hal::rmt::Error {
    fn into(self) -> WS2812Error {
        WS2812Error { msg: format!("ESP RMT Error: {:?}", self) }
    }
}