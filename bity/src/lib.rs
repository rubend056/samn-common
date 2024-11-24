#![cfg_attr(not(feature = "std"), no_std)]

use errors::Discriminant;

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum Error {
	BufferOverflow,
	BufferUnderflow,
	MAX
}

impl Discriminant for Error {
	fn discriminant(&self) -> u8 {
		// SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
		// between `repr(C)` structs, each of which has the `u8` discriminant as its first
		// field, so we can read the discriminant without offsetting the pointer.
		unsafe { *<*const _>::from(self).cast::<u8>() }
	}
	fn discriminant_max() -> u8 {
			Error::MAX as u8
	}
}


pub struct BitWriter<'a> {
	buffer: &'a mut [u8],
	byte_pos: usize,
	bit_pos: u8,
}

impl<'a> BitWriter<'a> {
	pub fn new(buffer: &'a mut [u8]) -> Self {
		BitWriter {
			buffer,
			byte_pos: 0,
			bit_pos: 0,
		}
	}

	pub fn write_bits(&mut self, value: u32, bits: u8) -> Result<(), Error> {
		#[cfg(feature = "std")]
		if bits < 32 {
			let max = 2u32.pow(bits as u32);
			if value >= max {
				panic!("value {value} > what {bits} bit/s can hold");
			}
		}
		for i in (0..bits).rev() {
			let bit = (value >> i) & 1;
			if self.byte_pos >= self.buffer.len() {
				return Err(Error::BufferOverflow);
			}
			self.buffer[self.byte_pos] |= (bit as u8) << (7 - self.bit_pos);
			self.bit_pos += 1;
			if self.bit_pos == 8 {
				self.byte_pos += 1;
				self.bit_pos = 0;
			}
		}
		Ok(())
	}

	pub fn finalize(&mut self) -> usize {
		if self.bit_pos != 0 {
			self.byte_pos += 1;
		}
		self.byte_pos
	}
}

/// Helper struct for reading bits from a byte buffer.
pub struct BitReader<'a> {
	buffer: &'a [u8],
	byte_pos: usize,
	bit_pos: u8,
}

impl<'a> BitReader<'a> {
	pub fn new(buffer: &'a [u8]) -> Self {
		BitReader {
			buffer,
			byte_pos: 0,
			bit_pos: 0,
		}
	}

	pub fn read_bits(&mut self, bits: u8) -> Result<u32, Error> {
		let mut value = 0;
		for _ in 0..bits {
			if self.byte_pos >= self.buffer.len() {
				return Err(Error::BufferUnderflow);
			}
			let bit = (self.buffer[self.byte_pos] >> (7 - self.bit_pos)) & 1;
			value = (value << 1) | (bit as u32);
			self.bit_pos += 1;
			if self.bit_pos == 8 {
				self.byte_pos += 1;
				self.bit_pos = 0;
			}
		}
		Ok(value)
	}
	pub fn finalize(&mut self) -> usize {
		if self.bit_pos != 0 {
			self.byte_pos += 1;
		}
		self.byte_pos
	}
}

#[cfg(test)]
mod test {

	enum Limb {
		Temp(Option<u32>),
		Hum(u8)
	}
	impl Limb {

	}

	#[test]
	fn serialize_deserialize() {
			// nothing yet, waiting to implement 
	}
}