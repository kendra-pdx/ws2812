use alloc::vec::Vec;
use core::time::Duration;

use crossbeam::atomic::AtomicCell;
use esp_hal::{Blocking, rmt::*};
use esp_hal::{gpio::Level, time::Rate};

use crate::color::{ChannelOrder, ColorChannels};
use crate::{Symbol, WS2812, WS2812Error, bits::*};

enum ChannelStatus<Tx> {
    Ready(Tx),
    Busy,
}

type SenderTx<const CH: u8> = Channel<Blocking, ConstChannelAccess<Tx, CH>>;

pub struct EspWS2812<const CH: u8, const N_COLOR_CHANNELS: usize> {
    tx: AtomicCell<ChannelStatus<SenderTx<CH>>>,
    p1: u32,
    p0: u32,
    color_order: ChannelOrder<N_COLOR_CHANNELS>,
}

impl<const CH: u8, const N_COLOR_CHANNELS: usize> EspWS2812<CH, N_COLOR_CHANNELS> {
    pub fn new(
        tx: SenderTx<CH>,
        clk_freq: Rate,
        color_order: ChannelOrder<N_COLOR_CHANNELS>,
    ) -> Self {
        EspWS2812 {
            tx: AtomicCell::new(ChannelStatus::Ready(tx)),
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

impl<const CH: u8, const N_COLOR_CHANNELS: usize> WS2812<N_COLOR_CHANNELS>
    for EspWS2812<CH, N_COLOR_CHANNELS>
{
    fn write<Px: ColorChannels<u8, N_COLOR_CHANNELS>>(
        &self,
        pixels: impl Iterator<Item = Px>,
    ) -> Result<(), WS2812Error> {
        use core::iter;

        let end: u32 = PulseCode::empty();
        let data = pixels
            .flat_map(|px| px.channels(self.color_order))
            .flat_map(|channel| channel.to_bits())
            .map(|s| if s { self.p1 } else { self.p0 })
            .chain(iter::once(end))
            .collect::<Vec<_>>();

        if let ChannelStatus::Ready(tx) = self.tx.swap(ChannelStatus::Busy) {
            let transaction = tx.transmit(data.as_slice()).map_err(|e| e.into())?;

            let tx = transaction.wait().map_err(|(e, tx)| {
                // restore the channel to ready state before returning the error.
                self.tx.store(ChannelStatus::Ready(tx));
                e.into()
            })?;

            self.tx.store(ChannelStatus::Ready(tx));
            Ok(())
        } else {
            Err(WS2812Error::new(
                "tx channel unavailable and is probably being used by another task.",
            ))
        }
    }
}

impl<'m> Into<WS2812Error<'m>> for esp_hal::rmt::Error {
    fn into(self) -> WS2812Error<'m> {
        WS2812Error::new(format!("ESP RMT Error: {:?}", self))
    }
}
