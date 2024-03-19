use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Sensor {
  Battery {
    level: u8,
  },
}

#[derive(Serialize, Deserialize,Debug)]
pub enum Command {
  Info,
}

#[derive(Serialize, Deserialize,Debug)]
pub enum Response {
  Info{board_version: u8}
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


