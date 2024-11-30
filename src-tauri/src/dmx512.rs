/// dmx512 - a light abstraction over the serialport crate for DMX512 communication
use serialport::{
    SerialPort, Error
};

pub struct DMX512 {
    universe: Vec<u8>,
    port: Box<dyn SerialPort>,
}

pub trait DMX {
    fn new(port: &str) -> Result<Self, Error> where Self: Sized;
    fn set_channel(&mut self, channel: u8, value: u8) -> Result<(), Error>;
    fn get_channel(&self, channel: u8) -> Result<u8, Error>;
    fn send(&mut self) -> Result<(), Error>;
}

impl DMX for DMX512 {
    fn new(port: &str) -> Result<Self, Error> {
        let port = serialport::new(port, 250_000)
            .timeout(std::time::Duration::from_millis(100))
            .open()?;
        let universe = vec![0; 512];
        Ok(DMX512 { universe, port })
    }

    fn set_channel(&mut self, channel: u8, value: u8) -> Result<(), Error> {
        self.universe[channel as usize] = value;
        Ok(())
    }

    fn get_channel(&self, channel: u8) -> Result<u8, Error> {
        Ok(self.universe[channel as usize])
    }

    fn send(&mut self) -> Result<(), Error> {
        self.port.write(&self.universe)?;
        Ok(())
    }
}