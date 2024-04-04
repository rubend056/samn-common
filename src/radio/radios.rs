use super::{Payload, Radio};
use cc1101::Cc1101;
use core::fmt::Debug;
use embedded_hal::{digital::OutputPin, spi::SpiDevice};
use nrf24::NRF24L01;

impl<E: Debug, CE: OutputPin<Error = E>, SPI: SpiDevice<u8, Error = SPIE>, SPIE: Debug> Radio<nrf24::Error<SPIE>>
	for NRF24L01<E, CE, SPI>
{
	/// Send still waits for retransmissions to finish :(
	/// Maybe we can do a send that doesn't wait? no, for that
	/// we'd have to switch off acks + retransmissions.
	/// Or, we could just set retransmissions to 7 + 1ms in between.
	/// That gives us around 10ms transmission time, instead of 4ms*15= around 100ms transmission time on max settings.
	/// DONE! Have to test. WORKS!
	///
	/// Now it sets the tx address to the payload's if it has one
	fn transmit_(&mut self, payload: &Payload) -> Result<Option<bool>, nrf24::Error<SPIE>> {
		static mut LAST_ADDR: Option<u16> = None;
		if let Some(address) = payload.address() {
			if unsafe { LAST_ADDR }.map(|last| last != address).unwrap_or(true) {
				// Set the tx address
				let mut bytes = [0xe7u8; 5];
				let addr_bytes = address.to_be_bytes();
				bytes[3] = addr_bytes[0];
				bytes[4] = addr_bytes[1];
				self.set_tx_addr(&bytes).unwrap();
				unsafe {
					LAST_ADDR = Some(address);
				}
			}
		}
		// if let Some(address) = payload.address() {
		// 	let mut bytes = [0xe7u8; 5];
		// 	let addr_bytes = address.to_be_bytes();
		// 	bytes[3] = addr_bytes[0];
		// 	bytes[4] = addr_bytes[1];
		// 	self.set_tx_addr(&bytes).unwrap();
		// }

		Ok(Some(self.send(&payload.0)?))
	}

	/// Receive with irq should work well (fast) :)
	fn receive_<P: embedded_hal::digital::InputPin>(
		&mut self,
		packet_ready_pin: &mut P,
		rx_addresses: Option<&[u16]>,
	) -> nb::Result<Payload, nrf24::Error<SPIE>> {
		NRF24L01::receive_with_irq(self, packet_ready_pin).and_then(|mut buf| {
			// Make buffer 32 items long
			while buf.len() < 32 {
				buf.push(0u8).unwrap();
			}
			// Turn it into a payload
			let payload = Payload(buf.into_array().unwrap());
			// Discard payloads that aren't for this address
			if let (Some(address), Some(addresses)) = (payload.address(), rx_addresses) {
				if addresses.contains(&address) {
					nb::Result::Ok(payload)
				} else {
					nb::Result::Err(nb::Error::WouldBlock)
				}
			} else 
			{
				nb::Result::Ok(payload)
			}
		})
	}
	fn set_rx_filter(&mut self, rx_addresses: &[u16]) -> Result<(), nrf24::Error<SPIE>> {
		for (i, address) in rx_addresses.iter().enumerate() {
			if i > 5 {
				return Ok(());
			}
			if i < 2 {
				// For pipe numbers 0 and 1 we have 5 bytes to work with
				let mut addr = [0xe7u8; 5];
				let address_bytes = address.to_be_bytes();
				addr[3] = address_bytes[0];
				addr[4] = address_bytes[1];
				self.set_rx_addr(i, &addr)?;
			} else {
				// For pipes 2,3,4,5 only set the least siginificant byte
				self.set_rx_addr(i, &address.to_le_bytes()[0..1])?;
			}
		}
		Ok(())
	}
}

impl<SPI: SpiDevice<u8, Error = SpiE>, SpiE> Radio<cc1101::Error<SpiE>> for Cc1101<SPI> {
	/// Transmit should work well (fast), because there are no retrasmissions/acks
	/// This just sends the packet as is
	fn transmit_(&mut self, payload: &Payload) -> Result<Option<bool>, cc1101::Error<SpiE>> {
		self.transmit(&payload.0)?;
		Ok(None)
	}
	/// Receive with irq should work well (fast) :)
	fn receive_<P: embedded_hal::digital::InputPin>(
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
	fn set_rx_filter(&mut self, rx_addresses: &[u16]) -> Result<(), cc1101::Error<SpiE>> {
		if rx_addresses.len() > 0 {
			// Only set least significant byte
			self.set_address_filter(cc1101::AddressFilter::Device(rx_addresses[0].to_le_bytes()[0]))?;
		}
		Ok(())
	}
}
