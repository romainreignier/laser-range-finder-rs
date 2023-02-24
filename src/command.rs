pub enum Command {
    LaserOn,
    LaserOff,
    ReadState,
    StartMeasurement,
    StartMeasurementFast,
    StartMeasurementSlow,
    ReadVersion,
    SwitchOff,
}

impl Command {
    pub fn value(&self) -> u8 {
        match *self {
            Command::LaserOn => b'O',
            Command::LaserOff => b'C',
            Command::ReadState => b'S',
            Command::StartMeasurement => b'D',
            Command::StartMeasurementSlow => b'M',
            Command::StartMeasurementFast => b'F',
            Command::ReadVersion => b'V',
            Command::SwitchOff => b'X',
        }
    }
}
