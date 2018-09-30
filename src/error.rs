use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum BatteryError {
    IoError,
    ConversionError,
}

impl fmt::Display for BatteryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BatteryError::IoError => f.write_str("IO Error"),
            BatteryError::ConversionError => f.write_str("Conversion Error"),
        }
    }
}

impl error::Error for BatteryError {
    fn description(&self) -> &str {
        match *self {
            BatteryError::IoError => "Error reading battery files",
            BatteryError::ConversionError => "Unable to convert data",
        }
    }
}

impl From<io::Error> for BatteryError {
    fn from(_: io::Error) -> BatteryError {
        BatteryError::IoError
    }
}
