use core::fmt::Debug;

use crate::node::{Message, NodeId};
use crate::radio::*;
use embedded_hal::{delay::DelayNs, digital::InputPin};

use super::addr_to_nrf24_hq_pipe;

pub fn send_looking_for_network<E: Debug, R: Radio<E>>(
	radio: &mut R,
	node_id: NodeId,
	node_addr: u16,
) {
	// Send looking for network
	send_message(radio, Message::SearchingNetwork(node_id), node_addr);
}
pub fn send_message<E: Debug, R: Radio<E>>(
	radio: &mut R,
  message: Message,
	node_addr: u16,
) {
	// Send looking for network
	let mut data = [0u8; 32];
	let data_l = message
		.serialize_to_bytes(&mut data)
		.unwrap();
	radio
		.transmit(&Payload::new_with_addr_from_array(
			data,
			data_l,
			node_addr,
			addr_to_nrf24_hq_pipe(node_addr),
		))
		.unwrap();
}

pub fn check_for_messages_for_a_bit<E: Debug, R: Radio<E>, P: InputPin, D: DelayNs>(
	radio: &mut R,
	irq: &mut P,
	delay: &mut D,
) -> Option<Message> {
	check_for_payloads_for_a_bit(radio, irq, delay)
		.and_then(|payload| Message::deserialize_from_bytes(payload.data()).ok().map(|v| v.0))
}
pub fn check_for_payloads_for_a_bit<E: Debug, R: Radio<E>, P: InputPin, D: DelayNs>(
	radio: &mut R,
	irq: &mut P,
	delay: &mut D,
) -> Option<Payload> {
	radio.to_rx().unwrap();

	// >= 150 ms wait
	for _ in 0..u8::MAX {
		if let Ok(payload) = radio.receive(irq, None) {
			return Some(payload);
		}
		delay.delay_us(500);
	}
	None
}
