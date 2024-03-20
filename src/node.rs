use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Sensor {
  /// Battery level (in percentage 0-100)
  /// 
  /// For a 1.5v battery that has a range of 1 to 1.6v
  /// being read by a microcontroller ADC of 10bits with a 3.3v power supply
  /// The reading from ADC would be from ~300 to ~500, so `(adc - 300) / 2` would yield the percentage
  Battery(u8),
  /// Tempearature in farenheit
  Temperature(u16),
  /// Humidity in percentage
  Humidity(u8)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActuatorValue {
  /// An on/off light
  Light(bool),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Actuator {
  /// Id, because there can actuators of the same kind
  pub id: u8,
  /// Value
  pub value: ActuatorValue,
}
impl PartialEq for Actuator{
  fn eq(&self, other: &Self) -> bool {
      self.id == other.id
  }
}
impl Eq for Actuator {}

#[derive(Serialize, Deserialize,Debug)]
pub enum Command {
  Info,
  /// Set sensor reporting interval in ms
  ReportingInterval(u16),
  /// Set actuator
  SetActuator(Actuator)
}

#[derive(Serialize, Deserialize,Debug, Default)]
pub enum Response {
  Info{board_version: u8, actuators: Vec<Actuator, ACTUATORS_MAX>},
  #[default] Ok,
  ErrActuatorNotFound,
  ErrActuatorValueTypeDoesntMatch
}

pub const ACTUATORS_MAX:usize = 4;
pub const SENSORS_MAX:usize = 4;


#[derive(Serialize, Deserialize,Debug)]
pub enum MessageData {
  Command{
    id: u8,
    command: Command
  },
  Response{
    id: u16,
    id_c: u8,
    response: Response
  },
  SensorData{
    id: u16,
    data: Vec<Sensor, SENSORS_MAX>
  }
}


// #[derive(Serialize, Deserialize)]
// pub struct Message {
//   for_id: u16,
//   message: MessageData
// }


