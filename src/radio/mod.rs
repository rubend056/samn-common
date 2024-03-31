/// Provides a trait for Radios to implement, so that we only use 1 API

mod radios;

pub trait Radio<E> {
  fn transmit_(&mut self, payload:&Payload) -> Result<Option<bool>, E>;
  fn receive_<P: embedded_hal::digital::InputPin>(&mut self, packet_ready_pin: &mut P) -> nb::Result<Payload, E>;
}



#[derive(Default)]
pub struct Payload([u8; 32]);
impl Payload {
    /// Copies the data to the payload
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
    pub fn new_with_addr(data: &[u8], address: u8) -> Self {
        let mut s = Self::default();
        if data.len() > s.0.len() - 2 {
            panic!("Data too big for Payload");
        }
        s.0[0] = u8::try_from(data.len()).unwrap() | (1 << 7);
        s.0[1] = address;
        for (i, d) in data.iter().enumerate() {
            s.0[i + 2] = *d;
        }
        s
    }
    pub fn address(&self) -> Option<u8> {
        if (self.0[0] & (1 << 7)) > 0 {
            Some(self.0[1])
        } else {
            None
        }
    }
    /// Get the length of the packet
    pub fn len(&self) -> u8 {
        self.0[0] & !(1 << 7)
    }
    /// Get the data section of the packet
    pub fn data(&self) -> &[u8] {
        &self.0[1..(self.len() + 1) as usize]
    }
}