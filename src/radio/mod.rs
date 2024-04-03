/// Provides a trait for Radios to implement, so that we only use 1 API
mod radios;

pub trait Radio<E> {
	fn transmit_(&mut self, payload: &Payload) -> Result<Option<bool>, E>;
	fn receive_<P: embedded_hal::digital::InputPin>(&mut self, packet_ready_pin: &mut P) -> nb::Result<Payload, E>;
}

#[derive(Default)]
pub struct Payload([u8; 32]);
impl Payload {
	/// Copies the data to the payload appending the length as the first byte
	pub fn new(data: &[u8]) -> Self {
		let mut s = Self::default();
		if data.len() > s.0.len() - 1 {
			panic!("Data too big for Payload");
		}
		s.0[0] = u8::try_from(data.len()).unwrap();
		for (i, d) in data.iter().enumerate() {
			s.0[i + 1] = *d;
		}
		s
	}
	/// Copies the data to the payload appending the length as the first byte, and address as 2nd and 3rd bytes
	pub fn new_with_addr(data: &[u8], address: u16) -> Self {
		let mut s = Self::default();
		if data.len() > s.0.len() - 2 {
			panic!("Data too big for Payload");
		}
		s.0[0] = u8::try_from(data.len()).unwrap() | (1 << 7);
		let address_bytes = address.to_le_bytes();
		s.0[1] = address_bytes[0];
		s.0[2] = address_bytes[1];
		for (i, d) in data.iter().enumerate() {
			s.0[i + 3] = *d;
		}
		s
	}
	pub fn address(&self) -> Option<u16> {
		if (self.0[0] & (1 << 7)) > 0 {
			Some(u16::from_le_bytes(self.0[1..3].try_into().unwrap()))
		} else {
			None
		}
	}
	/// Get the length of the packet
	pub fn len(&self) -> usize {
		(self.0[0] & !(1 << 7)).into()
	}
	/// Get the data section of the packet
	pub fn data(&self) -> &[u8] {
		let header_length = if self.address().is_some() { 3 } else { 1 } as usize;
		&self.0[header_length..header_length + self.len()]
	}
}
