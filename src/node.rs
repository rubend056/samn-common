use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Board {
	/// For all samn boards <= 8
	SamnV8,
	SamnV9,
	SamnDC,
	SamnSwitch,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Sensor {
	/// Battery level (in percentage 0-100)
	///
	/// For a 1.5v battery that has a range of 1 to 1.6v
	/// being read by a microcontroller ADC of 10bits with a 3.3v power supply
	/// The reading from ADC would be from ~300 to ~500, so `(adc - 300) / 2` would yield the percentage
	Battery(u8),
	/// - Temperature in Celcius * 100
	/// - Humidity in percentage
	TempHum((i16, u8)),
}

impl core::ops::Add for Sensor {
	type Output = Sensor;
	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Battery(level), Self::Battery(level_in)) => Self::Battery((level + level_in) / 2),
			(Self::TempHum((temp, hum)), Self::TempHum((temp_in, hum_in))) => {
				Self::TempHum(((temp + temp_in) / 2, (hum + hum_in) / 2))
			}
			_ => panic!("Can't add two different sensors"),
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Actuator {
	/// An on/off light
	Light(bool),
}

impl core::ops::Add for Actuator {
	type Output = Actuator;
	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Light(on), Self::Light(on_in)) => Self::Light(on || on_in), // _ => panic!("Can't add two different actuators")
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LimbType {
	Sensor {
		/// The reporting interval (in seconds)
		/// We will report on the closest value after wakeup.
		report_interval: u16,
		data: Option<Sensor>,
	},
	Actuator(Actuator),
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Limb(pub LimbId, pub LimbType);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Command {
	/// Gets node Info
	Info,
	/// Get node Limb states
	Limbs,
	/// Set a limb
	SetLimb(Limb),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeInfo {
	pub board: Board,
	/// Heartbeat interval in seconds
	pub heartbeat_interval: u16,
}

pub const LIMBS_MAX: usize = 3;
pub type Limbs = [Option<Limb>;LIMBS_MAX];

#[repr(u8)]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum Response {
	#[default]
	Ok,
	Info(NodeInfo),
	Limbs(Limbs),
	Heartbeat(u32),
	ErrLimbNotFound = 200,
	ErrLimbTypeDoesntMatch,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageData {
	Command {
		/// What command id is this
		id: u8,
		command: Command,
	},
	Response {
		/// Which command id are we responding to
		id: Option<u8>,
		response: Response,
	},
}

pub type NodeId = u32;
pub type NodeAddress = u16;
pub type LimbId = u8;

/// A message handles data transport between application layer
#[derive(Serialize, Deserialize, Debug,Clone)]
pub enum Message {
	// A message
	Message(MessageData),

	/// Relay a message to this node_id
	RelayMessage(NodeId, MessageData),

	/// A node searching a network for itself
	/// 
	/// (node_id)
	SearchingNetwork(NodeId),

	/// An id has been given to this node 
	/// 
	/// (node_id, node_addr)
	Network(NodeId, NodeAddress),

	/// A debug message 
	/// 
	/// (node_id, message)
	DebugMessage(NodeId, [u8;20]),

	// /// A relay searching a network for this specific node
	// /// 
	// /// (relay_id, node_id)
	// RelaySearchingNetwork(NodeId, NodeId),
}
