use core::fmt::Debug;

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;
pub use embedded_nrf24l01::{CrcMode, DataRate, NRF24L01, Device};

pub fn init<E: Debug, CE: OutputPin<Error = E>, SPI: SpiDevice<u8, Error = SPIE>, SPIE: Debug>(
    nrf24: &mut NRF24L01<E, CE, SPI>,
) {
    nrf24.set_frequency(8).unwrap();
    nrf24.set_auto_retransmit(15, 15).unwrap();
    nrf24.set_rf(&DataRate::R2Mbps, 0).unwrap();
    nrf24
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .unwrap();
    nrf24
        .set_auto_ack(&[true, false, false, false, false, false])
        .unwrap();
    nrf24.set_pipes_rx_lengths(&[None; 6]).unwrap();
    nrf24.set_crc(CrcMode::TwoBytes).unwrap();
    nrf24.set_rx_addr(0, &b"fnord"[..]).unwrap();
    nrf24.set_tx_addr(&b"fnord"[..]).unwrap();
    nrf24.flush_rx().unwrap();
    nrf24.flush_tx().unwrap();
}