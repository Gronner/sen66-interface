//! Errors emitted by this library.

use embedded_hal::i2c;
use thiserror::Error;

/// Error variants emitted when interacting with the sensor.
#[derive(Debug, Error, PartialEq)]
pub enum Sen66Error<I2C: i2c::Error> {
    /// Emitted when an error handling the data has occurred.
    #[error(transparent)]
    DataError(#[from] DataError),
    /// Emitted when the sensor reports a failed forced CO2 recalibration.
    #[error("The forced CO2 recalibration has failed.")]
    FailedCo2Recalibration,
    /// Emitted when an error from the I2C bus has occurred.
    #[error(transparent)]
    I2cError(#[from] I2C),
    /// Emitted when the sensor has an set error flag.
    #[error(transparent)]
    DeviceError(#[from] DeviceError),
    /// Emitted when a command is called in the wrong operating state. Use
    /// [start_measurement](crate::asynch::Sen66::start_measurement) to
    /// enter the measuring State, use [stop_measurement](crate::asynch::Sen66::stop_measurement) to enter the idle state.
    #[error("Command called in invalid state: {0}")]
    WrongState(&'static str),
}

#[cfg(feature = "defmt")]
impl<I2C: i2c::Error> defmt::Format for Sen66Error<I2C> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", self)
    }
}

/// Error variants emitted when handling sensor data.
#[derive(Debug, Error, PartialEq)]
pub enum DataError {
    /// Emitted when the CRC check for received data fails.
    #[error("CRC check failed.")]
    CrcFailed,
    /// Emitted when a string is constructed that contains either non-ASCII values or no null
    /// terminator within its bounds.
    #[error("Received data is not a null-terminated ASCII string.")]
    NotASCIIString,
    /// Emitted when data received does not match the expected data size.
    #[error("Buffer size received to wrong size for expected data.")]
    ReceivedBufferWrongSize,
    /// Emitted when a enum value received is not within the expected value range. Could occur if
    /// the firmware of the sensor has received updates.
    #[error("Unexpected Value for {parameter}: expected {expected} got {actual}")]
    UnexpectedValueReceived {
        /// Name of the parameter
        parameter: &'static str,
        /// Description of the expected value range
        expected: &'static str,
        /// Actual value received
        actual: u16,
    },
    /// Emitted when a value is used to construct data send to the sensor, but the value is not in
    /// the specified value's range. Adjust the argument to a value within the specified bounds.
    #[error("{parameter} must be between {min} and {max} {unit}.")]
    ValueOutOfRange {
        /// Name of the parameter
        parameter: &'static str,
        /// Lower limit of the value
        min: i32,
        /// Upper limit of the value
        max: i32,
        /// Unit of the value
        unit: &'static str,
    },
}

#[cfg(feature = "defmt")]
impl defmt::Format for DataError {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", self)
    }
}

/// Encodes the error flags set in the [`DeviceStatusRegister`](crate::data::DeviceStatusRegister)
#[derive(Debug, Error, PartialEq)]
#[error(
    "Sensor has errors set:
    PM:  {pm}
    CO2: {co2}
    Gas: {gas}
    RHT: {rht}
    Fan: {fan}"
)]
pub struct DeviceError {
    /// PM sensor error present
    pub pm: bool,
    /// CO2 sensor error present
    pub co2: bool,
    /// Gas sensor error present
    pub gas: bool,
    /// RH/T sensor error present
    pub rht: bool,
    /// Fan error present
    pub fan: bool,
}
