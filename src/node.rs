use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Sensor {
  Battery {
    level: u8,
  },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActuatorValue {
  Light(bool),
}

#[derive(Serialize, Deserialize, Debug)]
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
  Info{board_version: u8},
  #[default] Ok,
  ErrActuatorNotFound,
  ErrActuatorValueTypeDoesntMatch
}


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


