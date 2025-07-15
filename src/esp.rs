use core::time::Duration;

use esp_hal::rmt::*;
use esp_hal::{gpio::Level, time::Rate};

use crate::color::{ChannelOrder, ColorChannels};
use crate::{Symbol, WS2812, WS2812Error, bits::*};

pub struct EspWS2812<Tx: TxChannel, const N_COLOR_CHANNELS: usize, const N_PX: usize> {
    tx: Tx,
    p1: u32,
    p0: u32,
    color_order: ChannelOrder<N_COLOR_CHANNELS>,
}

impl<Tx: TxChannel, const N_COLOR_CHANNELS: usize, const N_PX: usize> EspWS2812<Tx, N_COLOR_CHANNELS, N_PX> {
    pub fn new(tx: Tx, clk_freq: Rate, color_order: ChannelOrder<N_COLOR_CHANNELS>) -> Self {
        EspWS2812 {
            tx,
            p1: symbol_to_pulse_code(&clk_freq, &Symbol::T1),
            p0: symbol_to_pulse_code(&clk_freq, &Symbol::T0),
            color_order,
        }
    }
}

fn duration_to_ticks(frequency: &Rate, duration: &Duration) -> u16 {
    const NANOS_PER_SECOND: u64 = 1_000_000_000;

    let ticks_hz = frequency.as_hz() as u64;
    let duration_ns = u64::try_from(duration.as_nanos()).expect("Overflow in duration_to_ticks");

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

impl<Tx: TxChannel, const N_COLOR_CHANNELS: usize, const N_PX: usize> WS2812<N_COLOR_CHANNELS>
    for EspWS2812<Tx, N_COLOR_CHANNELS, N_PX>
{
    fn write<Px: ColorChannels<u8, N_COLOR_CHANNELS>>(
        self,
        pixels: impl Iterator<Item = Px>,
    ) -> Result<Self, WS2812Error> {
        use core::iter;

        let end: u32 = PulseCode::empty();
        let data = pixels
            .flat_map(|px| px.channels(self.color_order))
            .flat_map(|channel| channel.to_bits())
            .map(|s| if s { self.p1 } else { self.p0 })
            .chain(iter::once(end))
            .collect::<heapless::Vec<u32, N_PX>>();

        let tx = self.tx.transmit(data.as_slice()).map_err(|e| e.into())?;

        let tx: Tx = tx.wait().map_err(|(e, _)| e.into())?;

        Ok(EspWS2812 { tx, ..self })
    }
}

impl Into<WS2812Error> for esp_hal::rmt::Error {
    fn into(self) -> WS2812Error {
        WS2812Error {
            msg: format!("ESP RMT Error: {:?}", self),
        }
    }
}
