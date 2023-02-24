use crate::error::Error;
use core::fmt;
use core::str::{self, FromStr};
use core::write;

pub enum Fault {
    Unknown,
    // Should be >= 2.0 V
    VBatTooLow,
    InternalError,
    // < -20°C
    ModuleTemperatureTooLow,
    // > +40°C
    ModuleTemperatureTooHigh,
    TargetOutOfRange,
    InvalidMeasureResult,
    BackgroundLightTooStrong,
    LaserSignalTooWeak,
    LaserSignalTooStrong,
    HardwareFault,
    LaserSignalNotStable,
}

impl Fault {
    pub fn fault(error_code: u8) -> Fault {
        match error_code {
            1 => Fault::VBatTooLow,
            2 => Fault::InternalError,
            3 => Fault::ModuleTemperatureTooLow,
            4 => Fault::ModuleTemperatureTooHigh,
            5 => Fault::TargetOutOfRange,
            6 => Fault::InvalidMeasureResult,
            7 => Fault::BackgroundLightTooStrong,
            8 => Fault::LaserSignalTooWeak,
            9 => Fault::LaserSignalTooStrong,
            10 | 11 | 12 | 13 | 14 | 16 | 17 => Fault::HardwareFault,
            15 => Fault::LaserSignalNotStable,
            _ => Fault::Unknown,
        }
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Fault::Unknown => write!(f, "Unknown error"),
            Fault::VBatTooLow => write!(f, "VBat too low (<2.0V)"),
            Fault::InternalError => write!(f, "Internal Error"),
            Fault::ModuleTemperatureTooLow => write!(f, "Module temperature is too low (<-20°)"),
            Fault::ModuleTemperatureTooHigh => write!(f, "Module temperature is too high (>+40°)"),
            Fault::TargetOutOfRange => write!(f, "Target out of measure range"),
            Fault::InvalidMeasureResult => write!(f, "Invalid measure result"),
            Fault::BackgroundLightTooStrong => write!(f, "Background light is too weak"),
            Fault::LaserSignalTooWeak => write!(f, "Laser signal too weak"),
            Fault::LaserSignalTooStrong => write!(f, "Laser signal too strong"),
            Fault::HardwareFault => write!(f, "Hardware Fault"),
            Fault::LaserSignalNotStable => write!(f, "Laser signal not stable"),
        }
    }
}

pub struct Measure {
    pub distance: f32,
    pub quality: u32,
}

pub struct Version {
    pub serial: u32,
    pub version: u32,
}

pub struct State {
    pub temperature: f32,
    pub input_voltage: f32,
}

pub enum LaserResponse {
    OK,
    Fault(Fault),
    Measure(Measure),
    State(State),
    Version(Version),
}

pub fn parse_laser_response(output: &str) -> Result<LaserResponse, Error> {
    if output.starts_with(",OK!") {
        return Ok(LaserResponse::OK);
    }
    if output.starts_with(":Er") && output.len() >= 6 {
        let error_code = &output[3..5];
        match u8::from_str(error_code) {
            Ok(error_code) => {
                return Ok(LaserResponse::Fault(Fault::fault(error_code)));
            }
            Err(_) => return Err(Error::Parse),
        }
    }
    if output.contains("m") && output.len() >= 12 {
        let distance = &output[0..6];
        let quality = &output[8..12];
        match f32::from_str(distance) {
            Ok(distance) => match u32::from_str(quality) {
                Ok(quality) => return Ok(LaserResponse::Measure(Measure { distance, quality })),
                Err(_) => return Err(Error::Parse),
            },
            Err(_) => return Err(Error::Parse),
        }
    }
    if output.contains("C") && output.len() >= 11 {
        let temperature = &output[0..4];
        let input_voltage = &output[7..10];
        match f32::from_str(temperature) {
            Ok(temperature) => match f32::from_str(input_voltage) {
                Ok(input_voltage) => {
                    return Ok(LaserResponse::State(State {
                        temperature,
                        input_voltage,
                    }))
                }
                Err(_) => return Err(Error::Parse),
            },
            Err(_) => return Err(Error::Parse),
        }
    }
    if output.len() >= 15 {
        let serial = &output[0..10];
        let version = &output[10..15];
        match u32::from_str(serial) {
            Ok(serial) => match u32::from_str(version) {
                Ok(version) => return Ok(LaserResponse::Version(Version { serial, version })),
                Err(_) => return Err(Error::Parse),
            },
            Err(_) => return Err(Error::Parse),
        }
    }
    return Err(Error::Parse);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_version() {
        let v = parse_laser_response("170225002929456");
        assert!(v.is_ok());
        match v.unwrap() {
            LaserResponse::Version(version) => {
                assert_eq!(version.serial, 1702250029);
                assert_eq!(version.version, 29456);
            }
            _ => panic!("Bad response type"),
        }
    }

    #[test]
    fn can_parse_ok() {
        let v = parse_laser_response(",OK!");
        assert!(v.is_ok());
        assert!(matches!(v.unwrap(), LaserResponse::OK));
    }

    #[test]
    fn can_parse_state() {
        let v = parse_laser_response("18.0'C,3.0V");
        assert!(v.is_ok());
        match v.unwrap() {
            LaserResponse::State(state) => {
                assert_eq!(state.temperature, 18.0);
                assert_eq!(state.input_voltage, 3.0);
            }
            _ => panic!("Bad response type"),
        }
    }

    #[test]
    fn can_parse_measure() {
        let v = parse_laser_response("12.345m,0079");
        assert!(v.is_ok());
        match v.unwrap() {
            LaserResponse::Measure(measure) => {
                assert_eq!(measure.distance, 12.345);
                assert_eq!(measure.quality, 79);
            }
            _ => panic!("Bad response type"),
        }
    }

    #[test]
    fn can_parse_fault() {
        let v = parse_laser_response(":Er01!");
        assert!(v.is_ok());
        match v.unwrap() {
            LaserResponse::Fault(fault) => {
                assert!(matches!(fault, Fault::VBatTooLow));
            }
            _ => panic!("Bad response type"),
        }
    }

    #[test]
    fn cannot_parse_version_too_short() {
        let v = parse_laser_response("17022500292945");
        assert!(v.is_err());
    }

    #[test]
    fn cannot_parse_ok_too_short() {
        let v = parse_laser_response(",OK");
        assert!(v.is_err());
    }

    #[test]
    fn cannot_parse_state_too_short() {
        let v = parse_laser_response("18.0'C,3.0");
        assert!(v.is_err());
    }

    #[test]
    fn cannot_parse_measure_too_short() {
        let v = parse_laser_response("12.345,0079");
        assert!(v.is_err());
    }

    #[test]
    fn cannot_parse_fault_too_short() {
        let v = parse_laser_response(":Er01");
        assert!(v.is_err());
    }
}
