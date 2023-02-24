#![cfg_attr(not(test), no_std)]

use crate::command::Command;
use crate::error::Error;
use core::str;
use embedded_hal::serial;
use embedded_hal::timer;
use fugit::MicrosDurationU64;
use heapless::Vec;

pub mod command;
pub mod error;
pub mod response;

pub struct LaserRangeFinder<S, T> {
    serial: S,
    timer: T,
}

impl<S, T> LaserRangeFinder<S, T>
where
    S: serial::Read<u8> + serial::Write<u8>,
    T: timer::CountDown,
{
    pub fn new(serial: S, timer: T) -> Self {
        Self { serial, timer }
    }

    fn send_command(&mut self, cmd: Command) -> Result<(), Error> {
        self.serial.write(cmd.value()).map_err(|_| Error::Write)
    }

    pub fn read_response(
        &mut self,
        timeout: MicrosDurationU64,
    ) -> Result<response::LaserResponse, Error>
    where
        <T as timer::CountDown>::Time: From<fugit::MicrosDurationU64>,
    {
        self.timer.start(timeout);

        let mut buffer: Vec<u8, 32> = Vec::new();
        loop {
            match self.serial.read() {
                Ok(byte) => {
                    if buffer.push(byte).is_err() {
                        return Err(Error::BufferFull);
                    }

                    // End of line
                    if byte == b'\n' {
                        return response::parse_laser_response(str::from_utf8(&buffer).unwrap());
                    }
                }
                Err(nb::Error::WouldBlock) => (),
                Err(_) => return Err(Error::Read),
            }
            // Check time
            match self.timer.wait() {
                Ok(()) => return Err(Error::Timeout),
                Err(nb::Error::WouldBlock) => (),
                Err(_) => return Err(Error::Timer),
            }
        }
    }

    pub fn laser_on(&mut self) -> Result<(), crate::error::Error> {
        self.send_command(Command::LaserOn)
    }

    pub fn laser_off(&mut self) -> Result<(), crate::error::Error> {
        self.send_command(Command::LaserOff)
    }

    pub fn start_measure(&mut self) -> Result<(), crate::error::Error> {
        self.send_command(Command::StartMeasurement)
    }

    pub fn start_measure_slow(&mut self) -> Result<(), crate::error::Error> {
        self.send_command(Command::StartMeasurementSlow)
    }

    pub fn start_measure_fast(&mut self) -> Result<(), crate::error::Error> {
        self.send_command(Command::StartMeasurementFast)
    }
}
