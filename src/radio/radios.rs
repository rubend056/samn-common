use super::{Payload, Radio};
use cc1101::Cc1101;
use core::fmt::Debug;
use embedded_hal::{digital::OutputPin, spi::SpiDevice};
use nrf24::NRF24L01;

impl<
		E: Debug,
		CE: OutputPin<Error = E>,
		SPI: SpiDevice<u8, Error = SPIE>,
		SPIE: Debug
	> Radio<nrf24::Error<SPIE>> for NRF24L01<E, CE, SPI>
{
	/// Send still waits for retransmissions to finish :(
	/// Maybe we can do a send that doesn't wait? no, for that 
	/// we'd have to switch off acks + retransmissions.
	/// Or, we could just set retransmissions to 7 + 1ms in between.
	/// That gives us around 10ms transmission time, instead of 4ms*15= around 100ms transmission time on max settings.
	/// DONE! Have to test. WORKS
	fn transmit_(&mut self, payload: &Payload) -> Result<Option<bool>, nrf24::Error<SPIE>> {
		
		Ok(Some(self.send(&payload.0)?))
	}

	/// Receive with irq should work well (fast) :)
	fn receive_<P: embedded_hal::digital::InputPin>(&mut self, packet_ready_pin: &mut P) -> nb::Result<Payload, nrf24::Error<SPIE>> {
		NRF24L01::receive_with_irq(self, packet_ready_pin).map(|mut buf| {
			// We make a new buffer because nrf24 can handle the variable packets
			
			// Make buffer 32 items long
			while buf.len() < 32 {buf.push(0u8).unwrap();}
			// Turn it into a payload
      Payload(buf.into_array().unwrap())
		})
	}
}

impl<SPI: SpiDevice<u8, Error = SpiE>, SpiE>
	Radio<cc1101::Error<SpiE>> for Cc1101<SPI>
{
	/// Transmit should work well (fast), because there are no retrasmissions/acks
	fn transmit_(&mut self, payload: &Payload) -> Result<Option<bool>, cc1101::Error<SpiE>> {
		self.transmit(&payload.0)?;
		Ok(None)
	}
	/// Receive with irq should work well (fast) :)
	fn receive_<P: embedded_hal::digital::InputPin>(&mut self, packet_ready_pin: &mut P) -> nb::Result<Payload, cc1101::Error<SpiE>> {
		Ok(Payload(self.receive(packet_ready_pin)?))
	}
}
