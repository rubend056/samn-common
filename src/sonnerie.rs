use std::io::Write;

use crate::node::{Limb, NodeInfo};
use sonnerie::{FromRecord, ToRecord};

impl ToRecord for &Limb {
	fn store(&self, buf: &mut std::vec::Vec<u8>) {
		buf.write_all(&postcard::to_vec::<_, 32>(self).unwrap()).unwrap();
	}
	fn format_char(&self) -> u8 {
		b'L'
	}
	fn size(&self) -> usize {
		32
	}
	fn variable_size(&self) -> bool {
		false
	}
}
impl ToRecord for &NodeInfo {
	fn store(&self, buf: &mut std::vec::Vec<u8>) {
		buf.write_all(&postcard::to_vec::<_, 32>(self).unwrap()).unwrap();
	}
	fn format_char(&self) -> u8 {
		b'N'
	}
	fn size(&self) -> usize {
		32
	}
	fn variable_size(&self) -> bool {
		false
	}
}

impl FromRecord<'_> for Limb {
	fn get(fmt_char: u8, bytes: &[u8]) -> std::io::Result<Self> {
		if fmt_char != b'L' {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("cannot decode Limb from '{}'", fmt_char as char),
			));
		}
		Ok(postcard::from_bytes(bytes).unwrap())
	}
}

impl FromRecord<'_> for NodeInfo {
	fn get(fmt_char: u8, bytes: &[u8]) -> std::io::Result<Self> {
		if fmt_char != b'N' {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("cannot decode NodeInfo from '{}'", fmt_char as char),
			));
		}
		Ok(postcard::from_bytes(bytes).unwrap())
	}
}
