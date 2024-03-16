use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Sensor {
  Battery {
    level: u8,
  },
}

#[derive(Serialize, Deserialize)]
pub enum Command {
  Info,
}

#[derive(Serialize, Deserialize)]
pub struct NodeInfo {
  pub id: u16,
  pub board_version: u8,
}

#[derive(Serialize, Deserialize)]
pub enum MessageData {
  Command(Command),
  SensorData(Vec<Sensor, 4>),
  NodeInfo(NodeInfo)
}

// #[derive(Serialize, Deserialize)]
// pub struct Message {
//   for_id: u16,
//   message: MessageData
// }


