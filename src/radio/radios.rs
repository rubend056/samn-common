use crate::radio::DEFAULT_PIPE;

use super::{Payload, Radio};
#[cfg(feature = "cc1101")]
use cc1101::Cc1101;
use embedded_hal::{digital::OutputPin, spi::SpiDevice};
#[cfg(feature = "nrf24")]
use nrf24::NRF24L01;

#[cfg(feature = "nrf24")]
impl<SPI: SpiDevice<u8>, CE: OutputPin> Radio<nrf24::Error<SPI::Error, CE::Error>>
	for NRF24L01<SPI, CE>
{
	
	fn init<D: embedded_hal::delay::DelayNs>(
		&mut self,
		delay: &mut D,
	) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.initialize(delay)?;
		self.configure()?;

		// Enable first 2 pipes
		const PIPES: [bool; 6] = [true, true, false, false, false, false];
		self.set_auto_ack_pipes(&PIPES)?;
		self.set_rx_enabled_pipes(&PIPES)?;
		self.set_dynamic_payload_pipes(&PIPES)?;
		Ok(())
	}

	fn transmit_start<D: embedded_hal::delay::DelayNs>(
		&mut self,
		payload: &Payload,
		delay: &mut D,
	) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.to_tx()?;

		// Set tx & rx0 to  pipe #
		static mut LAST_PIPE: u8 = 0;
		let pipe = payload.pipe();
		if unsafe { LAST_PIPE } != pipe {
			// Set the tx address
			let addr = [pipe, DEFAULT_PIPE, DEFAULT_PIPE, DEFAULT_PIPE, DEFAULT_PIPE];
			self.set_tx_addr(&addr)?;
			self.set_rx_addr(0, &addr)?;
			unsafe {
				LAST_PIPE = pipe;
			}
		}

		self.transmission_start(payload.payload(), nrf24::PayloadType::Payload, delay)?;
		Ok(())
	}
	fn transmit_poll(&mut self) -> nb::Result<bool, nrf24::Error<SPI::Error, CE::Error>> {
		self.transmission_ended()
	}

	/// Receive with irq should work well (fast) :)
	///
	/// Had to turn off irq, because pin would go low on first packet read
	/// Leaving other packets in the FIFO unread. This should be fixed now...
	fn receive<P: embedded_hal::digital::InputPin>(
		&mut self,
		_: &mut P,
		rx_addresses: Option<&[u16]>,
	) -> nb::Result<Payload, nrf24::Error<SPI::Error, CE::Error>> {

		if let Some((_, buf)) = self.receive_maybe()? {
			let payload = Payload(buf);
			// Discard payloads that aren't for this address
			if let (Some(address), Some(addresses)) = (payload.address(), rx_addresses) {
				if addresses.contains(&address) {
					return nb::Result::Ok(payload);
				}
			} else {
				return nb::Result::Ok(payload);
			}
		}
		nb::Result::Err(nb::Error::WouldBlock)
	}
	fn set_rx_filter(
		&mut self,
		rx_pipes: &[u8],
	) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		for (i, pipe) in rx_pipes.iter().enumerate() {
			if i <= 1 {
				self.set_rx_addr(
					i as u8,
					&[
						*pipe,
						DEFAULT_PIPE,
						DEFAULT_PIPE,
						DEFAULT_PIPE,
						DEFAULT_PIPE,
					],
				)?;
			} else if i <= 5 {
				self.set_rx_addr(i as u8, &[*pipe])?;
			}
		}
		Ok(())
	}
	fn to_tx(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.tx()
	}
	fn to_rx(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.rx()
	}
	fn to_idle(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.idle()
	}
	fn flush_rx(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
			self.flush_rx()
	}
	fn flush_tx(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
			self.flush_tx()
	}

	// Async function on nrf24 go straight to normal functions since they're non-blocking.
	#[cfg(feature = "tokio")]
	async fn to_rx_async(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.to_rx()
	}
	#[cfg(feature = "tokio")]
	async fn to_idle_async(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.to_idle()
	}
	#[cfg(feature = "tokio")]
	async fn to_tx_async(&mut self) -> Result<(), nrf24::Error<SPI::Error, CE::Error>> {
		self.to_tx()
	}
}

#[cfg(feature = "cc1101")]
impl<SPI: SpiDevice<u8, Error = SpiE>, SpiE> Radio<cc1101::Error<SpiE>> for Cc1101<SPI> {
	fn init<D: embedded_hal::delay::DelayNs>(
		&mut self,
		delay: &mut D,
	) -> Result<(), cc1101::Error<SpiE>> {
		delay.delay_ms(10);
		self.reset()?;
		delay.delay_ms(10);
		self.configure()?;
		self.flush_rx()?;
		self.flush_tx()?;
		Ok(())
	}
	/// Transmit should work well (fast), because there are no retrasmissions/acks
	/// This just sends the packet as is
	fn transmit_start<D: embedded_hal::delay::DelayNs>(
		&mut self,
		payload: &Payload,
		_: &mut D,
	) -> Result<(), cc1101::Error<SpiE>> {
		self.transmit_start(&payload.0)
	}
	fn transmit_poll(&mut self) -> nb::Result<bool, cc1101::Error<SpiE>> {
		self.transmit_poll().map(|_| true)
	}
	/// Receive with irq should work well (fast) :)
	fn receive<P: embedded_hal::digital::InputPin>(
		&mut self,
		packet_ready_pin: &mut P,
		rx_addresses: Option<&[u16]>,
	) -> nb::Result<Payload, cc1101::Error<SpiE>> {
		self.receive(packet_ready_pin).and_then(|buf| {
			// Turn it into a payload
			let payload = Payload(buf);
			// Discard payloads that aren't for this address
			if let (Some(address), Some(addresses)) = (payload.address(), rx_addresses) {
				if addresses.contains(&address) {
					nb::Result::Ok(payload)
				} else {
					nb::Result::Err(nb::Error::WouldBlock)
				}
			} else {
				nb::Result::Ok(payload)
			}
		})
	}
	fn set_rx_filter(&mut self, rx_pipes: &[u8]) -> Result<(), cc1101::Error<SpiE>> {
		if !rx_pipes.is_empty() {
			// Only set first byte
			self.set_address_filter(cc1101::AddressFilter::Device(rx_pipes[0]))?;
		}
		Ok(())
	}
	fn to_tx(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.to_tx()
	}
	fn to_rx(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.to_rx()
	}
	fn to_idle(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.to_idle()
	}
	fn flush_rx(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.flush_rx()
	}
	fn flush_tx(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.flush_tx()
	}

	// Implement async functions with underlying asyncs
	#[cfg(feature = "tokio")]
	async fn to_tx_async(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.to_tx_async().await
	}
	#[cfg(feature = "tokio")]
	async fn to_rx_async(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.to_rx_async().await
	}
	#[cfg(feature = "tokio")]
	async fn to_idle_async(&mut self) -> Result<(), cc1101::Error<SpiE>> {
		self.to_idle_async().await
	}
}
