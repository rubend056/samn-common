#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum NodeBitsError {
	BufferOverflow,
	BufferUnderflow,
	InvalidBoardCode,
	InvalidSensorCode,
	InvalidActuatorCode,
	InvalidCommandCode,
	InvalidResponseCode,
	InvalidMessageCode,
  InvalidMessageVersion,
}
pub type NodeBitsResult<T> = Result<T, NodeBitsError>;

pub type NodeId = u32;
pub type NodeAddress = u16;
/// max 16 (4 bits)
pub type LimbId = u8;

pub const LIMBS_MAX: usize = 3;
// pub type Limbs = Vec<Limb, 3>;
pub type Limbs = [Option<Limb>; 3];

/// Helper struct for writing bits into a byte buffer.
struct BitWriter<'a> {
	buffer: &'a mut [u8],
	byte_pos: usize,
	bit_pos: u8,
}

impl<'a> BitWriter<'a> {
	fn new(buffer: &'a mut [u8]) -> Self {
		BitWriter {
			buffer,
			byte_pos: 0,
			bit_pos: 0,
		}
	}

	fn write_bits(&mut self, value: u32, bits: u8) -> NodeBitsResult<()> {
		for i in (0..bits).rev() {
			let bit = (value >> i) & 1;
			if self.byte_pos >= self.buffer.len() {
				return Err(NodeBitsError::BufferOverflow);
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

	fn finalize(&mut self) -> usize {
		if self.bit_pos != 0 {
			self.byte_pos += 1;
		}
		self.byte_pos
	}
}

/// Helper struct for reading bits from a byte buffer.
struct BitReader<'a> {
	buffer: &'a [u8],
	byte_pos: usize,
	bit_pos: u8,
}

impl<'a> BitReader<'a> {
	fn new(buffer: &'a [u8]) -> Self {
		BitReader {
			buffer,
			byte_pos: 0,
			bit_pos: 0,
		}
	}

	fn read_bits(&mut self, bits: u8) -> NodeBitsResult<u32> {
		let mut value = 0;
		for _ in 0..bits {
			if self.byte_pos >= self.buffer.len() {
				return Err(NodeBitsError::BufferUnderflow);
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
	fn finalize(&mut self) -> usize {
		if self.bit_pos != 0 {
			self.byte_pos += 1;
		}
		self.byte_pos
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Board {
	/// For all samn boards <= 8
	SamnV8,
	SamnV9,
	SamnDC,
	SamnSwitch,
}

impl Board {
	fn from_code(code: u8) -> NodeBitsResult<Self> {
		match code {
			0 => Ok(Board::SamnV8),
			1 => Ok(Board::SamnV9),
			2 => Ok(Board::SamnDC),
			3 => Ok(Board::SamnSwitch),
			_ => Err(NodeBitsError::InvalidBoardCode),
		}
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct NodeInfo {
	pub board: Board,
	/// Heartbeat interval in seconds, max
	pub heartbeat_interval: u16,
}

impl NodeInfo {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		// Serialize board (2 bits, assuming up to 4 variants)
		let board_code = self.board_code();
		writer.write_bits(board_code as u32, 2)?;

		// Serialize heartbeat_interval (16 bits)
		writer.write_bits(self.heartbeat_interval as u32, 16)?;

		Ok(())
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<Self> {
		// Read board code (2 bits)
		let board_code = reader.read_bits(2)? as u8;
		let board = Board::from_code(board_code)?;

		// Read heartbeat_interval (16 bits)
		let heartbeat_interval = reader.read_bits(16)? as u16;

		Ok(NodeInfo {
			board,
			heartbeat_interval,
		})
	}

	fn board_code(&self) -> u8 {
		match self.board {
			Board::SamnV8 => 0,
			Board::SamnV9 => 1,
			Board::SamnDC => 2,
			Board::SamnSwitch => 3,
		}
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Sensor {
	/// Battery level (in percentage 0-100)
	Battery(u8),
	/// - Temperature in Celsius * 100
	/// - Humidity in percentage
	TempHum((i16, u8)),
	/// Current in mA
	Current(u16),
}

impl Sensor {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		// Write sensor code (4 bits)
		let sensor_code = self.sensor_code();
		writer.write_bits(sensor_code as u32, 4)?;

		match self {
			Sensor::Battery(level) => {
				// Write level (8 bits)
				writer.write_bits(*level as u32, 8)?;
			}
			Sensor::TempHum((temp, hum)) => {
				// Write temperature (16 bits)
				writer.write_bits(*temp as u32, 16)?;
				// Write humidity (8 bits)
				writer.write_bits(*hum as u32, 8)?;
			}
			Sensor::Current(current) => {
				// Write current (16 bits)
				writer.write_bits(*current as u32, 16)?;
			}
		}
		Ok(())
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<Self> {
		// Read sensor code (4 bits)
		let sensor_code = reader.read_bits(4)? as u8;

		match sensor_code {
			0 => {
				// Battery
				let level = reader.read_bits(8)? as u8;
				Ok(Sensor::Battery(level))
			}
			1 => {
				// TempHum
				let temp = reader.read_bits(16)? as i16;
				let hum = reader.read_bits(8)? as u8;
				Ok(Sensor::TempHum((temp, hum)))
			}
			2 => {
				// Current
				let current = reader.read_bits(16)? as u16;
				Ok(Sensor::Current(current))
			}
			_ => Err(NodeBitsError::InvalidSensorCode),
		}
	}

	fn sensor_code(&self) -> u8 {
		match self {
			Sensor::Battery(_) => 0,
			Sensor::TempHum(_) => 1,
			Sensor::Current(_) => 2,
			// Add other variants and codes here, up to 16
		}
	}
}

/// Max 16 Variants
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Actuator {
	/// An on/off light
	Light(bool),
}

impl Actuator {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		// Write actuator code (4 bits)
		let actuator_code = self.actuator_code();
		writer.write_bits(actuator_code as u32, 4)?;

		match self {
			Actuator::Light(on) => {
				// Write on/off bit (1 bit)
				writer.write_bits(if *on { 1 } else { 0 }, 1)?;
			}
		}
		Ok(())
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<Self> {
		// Read actuator code (4 bits)
		let actuator_code = reader.read_bits(4)? as u8;

		match actuator_code {
			0 => {
				// Light
				let on = reader.read_bits(1)? == 1;
				Ok(Actuator::Light(on))
			}
			_ => Err(NodeBitsError::InvalidActuatorCode),
		}
	}

	fn actuator_code(&self) -> u8 {
		match self {
			Actuator::Light(_) => 0,
			// Add other variants and codes here, up to 16
		}
	}
}

/// Max 2 Variants
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum LimbType {
	Sensor {
		/// The reporting interval (in seconds)
		report_interval: u16,
		data: Option<Sensor>,
	},
	Actuator(Actuator),
}

impl LimbType {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		match self {
			LimbType::Sensor { report_interval, data } => {
				// Write is_sensor bit (1)
				writer.write_bits(1, 1)?;
				// Write report_interval (16 bits)
				writer.write_bits(*report_interval as u32, 16)?;
				// Write data presence bit (1 bit)
				if let Some(sensor) = data {
					writer.write_bits(1, 1)?;
					// Serialize Sensor
					sensor.serialize_to_bits(writer)?;
				} else {
					writer.write_bits(0, 1)?;
				}
			}
			LimbType::Actuator(actuator) => {
				// Write is_sensor bit (0)
				writer.write_bits(0, 1)?;
				// Serialize Actuator
				actuator.serialize_to_bits(writer)?;
			}
		}
		Ok(())
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<Self> {
		// Read is_sensor bit (1 bit)
		let is_sensor = reader.read_bits(1)? == 1;

		if is_sensor {
			// Read report_interval (16 bits)
			let report_interval = reader.read_bits(16)? as u16;
			// Read data presence bit (1 bit)
			let has_data = reader.read_bits(1)? == 1;
			let data = if has_data {
				Some(Sensor::deserialize_from_bits(reader)?)
			} else {
				None
			};
			Ok(LimbType::Sensor { report_interval, data })
		} else {
			// Deserialize Actuator
			let actuator = Actuator::deserialize_from_bits(reader)?;
			Ok(LimbType::Actuator(actuator))
		}
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Limb(pub LimbId, pub LimbType);

impl Limb {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		// Write LimbId (8 bits)
		writer.write_bits(self.0 as u32, 4)?;
		// Serialize LimbType
		self.1.serialize_to_bits(writer)?;
		Ok(())
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<Self> {
		// Read LimbId (8 bits)
		let limb_id = reader.read_bits(4)? as u8;
		// Deserialize LimbType
		let limb_type = LimbType::deserialize_from_bits(reader)?;
		Ok(Limb(limb_id, limb_type))
	}
}

/// Max 16 Variants
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Command {
	/// Gets node Info
	Info,
	/// Get node Limb states
	Limbs,
	/// Set a limb
	SetLimb(Limb),
	/// Toggle a limb
	ToggleLimb(LimbId),
}

impl Command {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		// Write command code (4 bits)
		let command_code = self.command_code();
		writer.write_bits(command_code as u32, 4)?;

		match self {
			Command::Info => Ok(()),
			Command::Limbs => Ok(()),
			Command::SetLimb(limb) => {
				limb.serialize_to_bits(writer)?;
				Ok(())
			}
			Command::ToggleLimb(limb_id) => {
				// Write limb_id (8 bits)
				writer.write_bits(*limb_id as u32, 4)?;
				Ok(())
			}
		}
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<Self> {
		// Read command code (4 bits)
		let command_code = reader.read_bits(4)? as u8;

		match command_code {
			0 => Ok(Command::Info),
			1 => Ok(Command::Limbs),
			2 => {
				// SetLimb
				let limb = Limb::deserialize_from_bits(reader)?;
				Ok(Command::SetLimb(limb))
			}
			3 => {
				// ToggleLimb
				let limb_id = reader.read_bits(4)? as u8;
				Ok(Command::ToggleLimb(limb_id))
			}
			_ => Err(NodeBitsError::InvalidCommandCode),
		}
	}

	fn command_code(&self) -> u8 {
		match self {
			Command::Info => 0,
			Command::Limbs => 1,
			Command::SetLimb(_) => 2,
			Command::ToggleLimb(_) => 3,
			// Add other variants and codes here, up to 16
		}
	}
}

/// Max 16 Variants
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[repr(u8)]
pub enum Response {
	Ok,
	Info(NodeInfo),
	Limbs(Limbs),
	Heartbeat(u32),
	ErrLimbNotFound=200,
	ErrLimbTypeDoesntMatch,
}

impl Response {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		// Write response code (up to 4 bits)
		let response_code = self.response_code();
		writer.write_bits(response_code as u32, 4)?;

		match self {
			Response::Ok => Ok(()),
			Response::Info(node_info) => {
				node_info.serialize_to_bits(writer)?;
				Ok(())
			}
			Response::Limbs(limbs) => {
				// let limbs_n = limbs.len();
				// writer.write_bits(limbs_n as u32, 4)?;
				// Serialize limbs array
				for limb_option in limbs.iter() {
					// limb.serialize_to_bits(writer)?;
					// Write presence bit
					if let Some(limb) = limb_option {
						writer.write_bits(1, 1)?;
						limb.serialize_to_bits(writer)?;
					} else {
						writer.write_bits(0, 1)?;
					}
				}
				Ok(())
			}
			Response::Heartbeat(timestamp) => {
				// Write timestamp (32 bits)
				writer.write_bits(*timestamp as u32, 32)?;
				Ok(())
			}
			Response::ErrLimbNotFound => Ok(()),
			Response::ErrLimbTypeDoesntMatch => Ok(()),
		}
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<Self> {
		// Read response code (up to 4 bits)
		let response_code = reader.read_bits(4)? as u8;

		match response_code {
			0 => Ok(Response::Ok),
			1 => {
				// Info
				let node_info = NodeInfo::deserialize_from_bits(reader)?;
				Ok(Response::Info(node_info))
			}
			2 => {
				// Limbs
				let mut limbs: Limbs = Default::default();
				// let limbs_n = reader.read_bits(4)? as u8;
				// for _ in 0..limbs_n {
				//     limbs.push(Limb::deserialize_from_bits(reader)?).ok();
				// }
				for limb_option in limbs.iter_mut() {
					// Read presence bit
					let has_limb = reader.read_bits(1)? == 1;
					if has_limb {
						let limb = Limb::deserialize_from_bits(reader)?;
						*limb_option = Some(limb);
					}
				}
				Ok(Response::Limbs(limbs))
			}
			3 => {
				// Heartbeat
				let timestamp = reader.read_bits(32)? as u32;
				Ok(Response::Heartbeat(timestamp))
			}
			200 => Ok(Response::ErrLimbNotFound),
			201 => Ok(Response::ErrLimbTypeDoesntMatch),
			_ => Err(NodeBitsError::InvalidResponseCode),
		}
	}

	fn response_code(&self) -> u8 {
		match self {
			Response::Ok => 0,
			Response::Info(_) => 1,
			Response::Limbs(_) => 2,
			Response::Heartbeat(_) => 3,
			Response::ErrLimbNotFound => 200,
			Response::ErrLimbTypeDoesntMatch => 201,
		}
	}
}

/// Max 2 Variants
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum MessageData {
	Command {
		/// What command id is this (6 bits)
		id: u8, // 0-63
		command: Command,
	},
	Response {
		/// Which command id are we responding to (6 bits)
		id: Option<u8>, // 0-63
		response: Response,
	},
}

impl MessageData {
	fn serialize_to_bits(&self, writer: &mut BitWriter) -> NodeBitsResult<()> {
		match self {
			MessageData::Command { id, command } => {
				// Write is_command bit (1)
				writer.write_bits(1, 1)?;
				// Write id (6 bits)
				writer.write_bits(*id as u32, 6)?;
				// Serialize command
				command.serialize_to_bits(writer)?;
			}
			MessageData::Response { id, response } => {
				// Write is_command bit (0)
				writer.write_bits(0, 1)?;
				// Write id (6 bits), use 0 if None
				let id_value = id.unwrap_or(0);
				writer.write_bits(id_value as u32, 6)?;
				// Serialize response
				response.serialize_to_bits(writer)?;
			}
		}
		Ok(())
	}

	fn deserialize_from_bits(reader: &mut BitReader) -> NodeBitsResult<(Self, usize)> {
		let start_byte = reader.byte_pos;

		// Read is_command bit (1 bit)
		let is_command = reader.read_bits(1)? == 1;
		// Read id (6 bits)
		let id = reader.read_bits(6)? as u8;

		if is_command {
			// Deserialize command
			let command = Command::deserialize_from_bits(reader)?;
			Ok((MessageData::Command { id, command }, reader.byte_pos - start_byte))
		} else {
			// Deserialize response
			let response = Response::deserialize_from_bits(reader)?;
			let id_option = if id == 0 { None } else { Some(id) };
			Ok((
				MessageData::Response {
					id: id_option,
					response,
				},
				reader.byte_pos - start_byte,
			))
		}
	}
}

/// Max 16 Variants
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Message {
	// A message
	Message(MessageData),

	/// Relay a message to this node_id
	RelayMessage(NodeId, MessageData),

	/// A node searching a network for itself
	///
	/// (node_id)
	SearchingNetwork(NodeId),

	/// An address has been given to this node
	///
	/// (node_id, node_addr)
	Network(NodeId, NodeAddress),

	/// A debug message
	///
	/// (node_id, message)
	DebugMessage(NodeId, [u8; 20]),
	// Add other variants here, up to 16
}

/// Up to 4 types of message versions coexisting (0 - 3   2 bits)
const MESSAGE_VERSION:u8 = 1;

impl Message {
	/// Serialize the Message into bytes, returns the number of bytes written.
	pub fn serialize_to_bytes(&self, buffer: &mut [u8]) -> NodeBitsResult<usize> {
		let mut writer = BitWriter::new(buffer);

    // Write the message version
    writer.write_bits(MESSAGE_VERSION as u32, 2)?;

		// Write the message code (4 bits)
		let message_code = self.message_code();
		writer.write_bits(message_code as u32, 4)?;

		// Serialize based on variant
		match self {
			Message::Message(data) => {
				data.serialize_to_bits(&mut writer)?;
			}
			Message::RelayMessage(node_id, data) => {
				// Serialize node_id (32 bits)
				writer.write_bits(*node_id, 32)?;
				data.serialize_to_bits(&mut writer)?;
			}
			Message::SearchingNetwork(node_id) => {
				// Serialize node_id (32 bits)
				writer.write_bits(*node_id, 32)?;
			}
			Message::Network(node_id, node_address) => {
				// Serialize node_id (32 bits)
				writer.write_bits(*node_id, 32)?;
				// Serialize node_address (16 bits)
				writer.write_bits(*node_address as u32, 16)?;
			}
			Message::DebugMessage(node_id, msg) => {
				// Serialize node_id (32 bits)
				writer.write_bits(*node_id, 32)?;
				// Serialize msg (20 bytes)
				for byte in msg.iter() {
					writer.write_bits(*byte as u32, 8)?;
				}
			}
		}

		Ok(writer.finalize())
	}

	/// Deserialize a Message from bytes, returns the Message and the number of bytes read.
	pub fn deserialize_from_bytes(buffer: &[u8]) -> NodeBitsResult<(Self, usize)> {
		let mut reader = BitReader::new(buffer);

    // Read the message version (2 bits)
    let message_version = reader.read_bits(2)? as u8;
    if message_version != MESSAGE_VERSION {
      return Err(NodeBitsError::InvalidMessageVersion);
    }

		// Read the message code (4 bits)
		let message_code = reader.read_bits(4)? as u8;

		let message = match message_code {
			0 => {
				// Message::Message
				let (data, _) = MessageData::deserialize_from_bits(&mut reader)?;
				Message::Message(data)
			}
			1 => {
				// Message::RelayMessage
				let node_id = reader.read_bits(32)?;
				let (data, _) = MessageData::deserialize_from_bits(&mut reader)?;
				Message::RelayMessage(node_id, data)
			}
			2 => {
				// Message::SearchingNetwork
				let node_id = reader.read_bits(32)?;
				Message::SearchingNetwork(node_id)
			}
			3 => {
				// Message::Network
				let node_id = reader.read_bits(32)?;
				let node_address = reader.read_bits(16)? as u16;
				Message::Network(node_id, node_address)
			}
			4 => {
				// Message::DebugMessage
				let node_id = reader.read_bits(32)?;
				let mut msg = [0u8; 20];
				for byte in msg.iter_mut() {
					*byte = reader.read_bits(8)? as u8;
				}
				Message::DebugMessage(node_id, msg)
			}
			_ => {
				return Err(NodeBitsError::InvalidMessageCode);
			}
		};

		Ok((
			message,
			reader.finalize(),
		))
	}

	fn message_code(&self) -> u8 {
		match self {
			Message::Message(_) => 0,
			Message::RelayMessage(_, _) => 1,
			Message::SearchingNetwork(_) => 2,
			Message::Network(_, _) => 3,
			Message::DebugMessage(_, _) => 4,
			// Add other variants and codes here, up to 16
		}
	}
}

impl core::ops::Add for Sensor {
	type Output = Sensor;
	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Battery(level), Self::Battery(level_in)) => Self::Battery((level + level_in) / 2),
			(Self::TempHum((temp, hum)), Self::TempHum((temp_in, hum_in))) => {
				Self::TempHum(((temp + temp_in) / 2, (hum + hum_in) / 2))
			}
			(Self::Current(i), Self::Current(i_in)) => Self::Current(((i as u32 + i_in as u32) / 2) as u16),
			_ => panic!("Can't add two different sensors"),
		}
	}
}

impl core::ops::Add for Actuator {
	type Output = Actuator;
	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Light(on), Self::Light(on_in)) => Self::Light(on || on_in), // _ => panic!("Can't add two different actuators")
		}
	}
}

impl core::ops::Add for LimbType {
	type Output = LimbType;
	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(
				Self::Sensor {
					report_interval: ri,
					data: d,
				},
				Self::Sensor {
					report_interval: ri_in,
					data: d_in,
				},
			) => Self::Sensor {
				report_interval: (ri + ri_in) / 2,
				data: match (d, d_in) {
					(None, None) => None,
					(None, Some(s)) => Some(s),
					(Some(s), None) => Some(s),
					(Some(s), Some(s_in)) => Some(s + s_in),
				},
			},
			(Self::Actuator(actuator), Self::Actuator(actuator_in)) => Self::Actuator(actuator + actuator_in),
			_ => panic!("Can't add two different Limb Types"),
		}
	}
}

#[test]
fn serialize_limbs_bits() {
	let message = Message::Message(MessageData::Response {
		id: Some(55),
		response: Response::Limbs([
			Some(Limb(
				0,
				LimbType::Sensor {
					report_interval: 300,
					data: Some(Sensor::TempHum((1000, 50))),
				},
			)),
			Some(Limb(
				2,
				LimbType::Sensor {
					report_interval: 300,
					data: Some(Sensor::TempHum((1000, 50))),
				},
			)),
			None,
		]),
	});
	let mut data = [0u8; 32];
	let data_l = message.serialize_to_bytes(&data).unwrap();
  let message_out = Message::deserialize_from_bytes(&data).unwrap().0;
  assert_eq!(message, message_out);
}

