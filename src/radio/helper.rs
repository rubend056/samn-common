/**
 * These are helpers to make node code simpler
 * 
 */

use crate::node::{Message, NodeSerializeError};
use crate::radio::*;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;
use embedded_hal::{delay::DelayNs, digital::InputPin};
use errors::Discriminant;
use nrf24::NRF24L01;

use super::addr_to_nrf24_hq_pipe;

// No debug for this, to prevent accidentally unwrapping it.
// #[derive(Debug)]
#[repr(u8)]
pub enum Error<E> {
	RadioError(E) = 0,
	SerializationError(NodeSerializeError),
	SendingTimedOut,
}
const ERROR_MAX: u8 = 10;

impl<E> From<E> for Error<E> {
	fn from(value: E) -> Self {
		Self::RadioError(value)
	}
}
impl<E: Discriminant> Discriminant for Error<E> {
	fn discriminant(&self) -> u8 {
		// SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
		// between `repr(C)` structs, each of which has the `u8` discriminant as its first
		// field, so we can read the discriminant without offsetting the pointer.
		match self {
			Self::SerializationError(err) => ERROR_MAX + err.discriminant(),
			Self::RadioError(err) => {
				ERROR_MAX + NodeSerializeError::discriminant_max() + err.discriminant()
			}
			_ => unsafe { *<*const _>::from(self).cast::<u8>() },
		}
	}
	fn discriminant_max() -> u8 {
		ERROR_MAX + E::discriminant_max() + NodeSerializeError::discriminant_max()
	}
}

type SendResult<E> = Result<bool, Error<E>>;
// pub fn send_looking_for_network<E, R: Radio<E>, D: DelayNs>(
// 	radio: &mut R,
// 	node_id: NodeId,
// 	node_addr: u16,
// 	delay: &mut D,
// ) -> SendResult<E> {
// 	// Send looking for network
// 	send_message(radio, Message::SearchingNetwork(node_id), node_addr, delay)
// }

/// For NRF24 we need to enable first pipe and disable it afterwards
/// 
/// This is a very specific method just for nodes with nrf24.
/// We enable first pipe cause its needed for an ACK from HQ.
/// But disable it afterwards because we don't want to receive 
/// messages from other nodes that are headed to HQ.
pub fn send_message_nrf24<SPI : SpiDevice,CE:OutputPin, D: DelayNs>(
	radio: &mut NRF24L01<SPI, CE>,
	message: Message,
	node_addr: u16,
	delay: &mut D,
) -> SendResult<nrf24::Error<SPI::Error, CE::Error>> {
	// Enable first pipe
	radio.set_rx_enabled_pipes(&[true,true,false,false,false,false])?;
	let result = send_message_(radio, message, node_addr, delay)?;
	// Disable first pipe
	radio.set_rx_enabled_pipes(&[false,true,false,false,false,false])?;
	Ok(result)
}


/// Send a message
/// 
/// Changed the name to underscore to prevent nrf nodes from building
/// until they've been changed to the right one up there ^
pub fn send_message_<E, R: Radio<E>, D: DelayNs>(
	radio: &mut R,
	message: Message,
	node_addr: u16,
	delay: &mut D,
) -> SendResult<E> {
	let mut data = [0u8; 32];
	let data_l = message
		.serialize_to_bytes(&mut data)
		.map_err(Error::SerializationError)?;

	send_payload(
		radio,
		&Payload::new_with_addr_from_array(
			data,
			data_l,
			node_addr,
			addr_to_nrf24_hq_pipe(node_addr),
		),
		delay,
	)
}


pub fn send_payload<E, R: Radio<E>, D: DelayNs>(
	radio: &mut R,
	payload: &Payload,
	delay: &mut D,
) -> SendResult<E> {
	radio.transmit_start(payload, delay)?;

	// 64 ms max wait
	for _ in 0..u8::MAX {
		match radio.transmit_poll() {
			nb::Result::Ok(success) => {
				return Ok(success);
			}
			nb::Result::Err(nb::Error::Other(err)) => return Err(err.into()),
			nb::Result::Err(nb::Error::WouldBlock) => {}
		}
		delay.delay_us(250);
	}
	Err(Error::SendingTimedOut)
}

pub fn check_for_messages_for_a_bit<E, R: Radio<E>, P: InputPin, D: DelayNs>(
	radio: &mut R,
	irq: &mut P,
	delay: &mut D,
) -> Result<Option<Message>, E> {
	check_for_payloads_for_a_bit(radio, irq, delay).map(|payload| {
		if let Some(payload) = payload {
			Message::deserialize_from_bytes(payload.data())
				.ok()
				.map(|(message, _)| message)
		} else {
			None
		}
	})
}
pub fn check_for_payloads_for_a_bit<E, R: Radio<E>, P: InputPin, D: DelayNs>(
	radio: &mut R,
	irq: &mut P,
	delay: &mut D,
) -> Result<Option<Payload>, E> {
	radio.to_rx()?;

	// 127 ms max wait
	for _ in 0..u8::MAX {
		match radio.receive(irq, None) {
			nb::Result::Ok(payload) => {
				return Ok(Some(payload));
			}
			nb::Result::Err(nb::Error::Other(err)) => return Err(err),
			nb::Result::Err(nb::Error::WouldBlock) => {}
		}
		delay.delay_us(500);
	}
	Ok(None)
}
