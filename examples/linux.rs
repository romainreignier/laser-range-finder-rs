use fugit::ExtU64;
use laser_range_finder::LaserRangeFinder;
use linux_embedded_hal::{
    serial_core::{self, SerialPort},
    Serial, SysTimer,
};
use std::io;
use std::io::prelude::*;
use std::path::Path;

const SETTINGS: serial_core::PortSettings = serial_core::PortSettings {
    baud_rate: serial_core::Baud19200,
    char_size: serial_core::Bits8,
    parity: serial_core::ParityNone,
    stop_bits: serial_core::Stop1,
    flow_control: serial_core::FlowNone,
};

fn main() {
    let mut serial = Serial::open(Path::new("/dev/ttyUSB0")).unwrap();
    serial.0.configure(&SETTINGS).unwrap();
    let timer = SysTimer::new();
    let laser = LaserRangeFinder::new(serial, timer);

    println!("Type Enter for a new measure");
    let stdin = io::stdin();
    for _line in stdin.lock().lines() {
        println!("New measure");
        laser.laser_on().unwrap();
        laser.read_response(1u64.secs()).unwrap();
        laser.start_measure_slow().unwrap();
        match laser.read_response(1u64.secs()).unwrap() {
            laser_range_finder::response::LaserResponse::Measure(m) => {
                println!("Distance: {} m", m.distance)
            }
            laser_range_finder::response::LaserResponse::Fault(f) => println!("Error: {}", f),
        }
    }
}
