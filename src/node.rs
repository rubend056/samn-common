use heapless::{String, Vec};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize,Clone,Debug)]
pub enum Board {
  /// For all samn boards <= 8
  SamnV8,
  SamnV9
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Actuator {
  /// An on/off light
  Light(bool),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LimbType{Sensor{
  /// The reporting interval (in seconds)
  /// We will report on the closest value after wakeup.
  report_interval: u16,
  data: Option<Sensor> 
}, Actuator(Actuator)}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Limb(pub u16,pub LimbType);

#[derive(Serialize, Deserialize,Debug)]
pub enum Command {
  /// Gets node Info
  Info,
  /// Get node Limb states
  Limbs,
  /// Set a limb
  SetLimb(Limb)
}

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct NodeInfo {
  pub name: String::<12>,
  pub board: Board
}

pub const LIMBS_MAX:usize = 4;
pub type Limbs = Vec<Limb, LIMBS_MAX>;

#[repr(u8)]
#[derive(Serialize, Deserialize,Debug, Default)]
pub enum Response {
  #[default] Ok,
  Info(NodeInfo),
  Limbs(Limbs),
  Heartbeat(u32),
  ErrLimbNotFound=200,
  ErrLimbTypeDoesntMatch
}


#[derive(Serialize, Deserialize,Debug)]
pub enum MessageData {
  Command{
    /// What command id is this
    id: u8,
    command: Command
  },
  Response{
    /// Which command id are we responding to
    id: Option<u8>,
    response: Response
  },
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
  /// From/for node id
  pub id: u16,
  pub data: MessageData
}



