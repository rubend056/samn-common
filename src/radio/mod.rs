/// Provides a trait for Radios to implement, so that we only use 1 API
mod radios;

pub const DEFAULT_PIPE: u8 = 0x97u8;
/// Gives the pipe this node addr will receive on
/// - Used by node to set rx_filter
/// - Used by hq to send message to node
pub fn addr_to_rx_pipe(addr: u16) -> u8 {
	// The - 128 is to make sure we are out of HQ's pipe range
	((addr.wrapping_sub(128)) % 256) as u8
}
/// - Used by node to send a message to HQ
pub fn addr_to_nrf24_hq_pipe(_: u16) -> u8 {
	// Because nrf24 on HQ not working with multiple pipes :(
	DEFAULT_PIPE
}
pub fn addr_to_cc1101_hq_pipe(_: u16) -> u8 {
	DEFAULT_PIPE
}

pub trait Radio<E> {
	/// Resets the device (if possible) and configures with default settings
	fn init<D: embedded_hal::delay::DelayNs>(&mut self, delay: &mut D) -> Result<(), E>;
	/// A blocking transmit, waits for tramission to finish.
	fn transmit(&mut self, payload: &Payload) -> Result<Option<bool>, E>;
	/// A non-blocking transmission, puts packet in FIFO and puts radio in transmit mode.
	fn transmit_start(&mut self, payload: &Payload) -> Result<(), E>;
	/// Polls the trasmission. Returning when tranmission was done.
	fn transmit_poll(&mut self) -> nb::Result<bool, E>;
	/// Implemented on nrf24 + cc1101
	///
	/// Doesn't check rx_addresses on nrf24 because nrf24 can check the full address on hardware
	fn receive<P: embedded_hal::digital::InputPin>(
		&mut self,
		packet_ready_pin: &mut P,
		rx_addresses: Option<&[u16]>,
	) -> nb::Result<Payload, E>;
	/// For the nrf24 this will set the 6 data pipe addresses ()
	/// For the cc1101 this will set the 1 address filter (to the least significant byte on the first address)
	fn set_rx_filter(&mut self, rx_pipes: &[u8]) -> Result<(), E>;
	fn to_rx(&mut self) -> Result<(), E>;
	fn to_tx(&mut self) -> Result<(), E>;
	fn to_idle(&mut self) -> Result<(), E>;

	fn flush_rx(&mut self) -> Result<(), E>;
	fn flush_tx(&mut self) -> Result<(), E>;
}

/// Payload is (pipe, len, addr1, addr0, ...data)
#[derive(Default)]
pub struct Payload([u8; 32]);
impl Payload {
	// /// Copies the data to the payload appending the length as the first byte
	// pub fn new(data: &[u8]) -> Self {
	// 	let mut s = Self::default();
	// 	if data.len() > s.0.len() - 2 {
	// 		panic!("Data too big for Payload");
	// 	}
	// 	s.0[0] = u8::try_from(data.len()).unwrap();
	// 	s.0[1] = DEFAULT_PIPE;
	// 	for (i, d) in data.iter().enumerate() {
	// 		s.0[i + 2] = *d;
	// 	}
	// 	s
	// }

	/// Copies the data to the payload appending the pipe as 1rst byte,
	/// length as 2nd byte, address as 3rd and 4th bytes.
	pub fn new_with_addr(data: &[u8], address: u16, pipe: u8) -> Self {
		let mut s = Self::default();
		if data.len() > s.0.len() - 4 {
			panic!("Data too big for Payload");
		}
		s.0[0] = pipe;
		s.0[1] = u8::try_from(data.len()).unwrap() | (1 << 7);
		let address_bytes = address.to_le_bytes();
		s.0[2] = address_bytes[0];
		s.0[3] = address_bytes[1];
		for (i, d) in data.iter().enumerate() {
			s.0[i + 4] = *d;
		}
		s
	}

	fn has_address(&self) -> bool {
		((self.0[1] >> 7) & 1) == 1
	}
	pub fn address(&self) -> Option<u16> {
		if self.has_address() {
			Some(u16::from_le_bytes(self.0[2..4].try_into().unwrap()))
		} else {
			None
		}
	}

	fn header_length(&self) -> usize {
		if self.has_address() {
			4
		} else {
			2
		}
	}
	/// Get the length of the packet
	pub fn len(&self) -> usize {
		(self.0[1] & !(1 << 7)).into()
	}
	/// Get the pipe
	pub fn pipe(&self) -> u8 {
		self.0[0]
	}
	pub fn len_is_valid(&self) -> bool {
		self.len() != 0 && self.len() <= self.0.len() - self.header_length()
	}
	/// Get the data section of the packet
	pub fn data(&self) -> &[u8] {
		let header_length = self.header_length();
		&self.0[header_length..header_length + self.len()]
	}
}

mod test {

	#[test]
	fn try_payload() {
		use crate::radio::Payload;

		let mut payload = Payload::new_with_addr(&[1, 2, 3], 0x5555, 0x22);
		assert_eq!(payload.data(), [1, 2, 3]);
		assert_eq!(payload.len(), 3);
		assert_eq!(payload.len_is_valid(), true);
		assert_eq!(payload.header_length(), 4);
		assert_eq!(payload.has_address(), true);
		assert_eq!(payload.pipe(), 0x22);
		assert_eq!(payload.address(), Some(0x5555));

		payload.0[1] = 32 | (1 << 7);
		assert_eq!(payload.len_is_valid(), false);
		payload.0[1] = 28 | (1 << 7);
		assert_eq!(payload.len_is_valid(), true);
		payload.0[1] = 29 | (1 << 7);
		assert_eq!(payload.len_is_valid(), false);
	}
}
