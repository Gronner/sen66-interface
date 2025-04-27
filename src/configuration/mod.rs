//! Data types for configuring the SEN66's operations.

mod temperature;
mod tuning;

use crate::{
    error::DataError,
    util::{check_deserialization, check_range},
};
pub use temperature::{TemperatureAcceleration, TemperatureOffset};
pub use tuning::{NoxTuning, VocTuning};

/// Target CO2 concentration after a forced CO2 recalibration in ppm.
pub struct TargetCO2Concentration(u16);

impl From<u16> for TargetCO2Concentration {
    fn from(value: u16) -> Self {
        TargetCO2Concentration(value)
    }
}

impl From<TargetCO2Concentration> for u16 {
    fn from(value: TargetCO2Concentration) -> Self {
        value.0
    }
}

/// CO2 correction value determined after forced CO2 recalibration (FRC).
/// Is set to `0xFFFF` if recalibration has failed.
pub struct Co2Correction(u16);

impl Co2Correction {
    /// Returns true if recalibration has failed.
    pub fn is_valid(&self) -> bool {
        self.0 != 0xFFFF
    }
}

impl TryFrom<&[u8]> for Co2Correction {
    type Error = DataError;

    /// Computes the correction value from the received data. Does not perform the computation if
    /// `0xFFFF` has been received, indicating a failed FRC.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 3)?;
        let value = u16::from_be_bytes([data[0], data[1]]);
        let value = if value != 0xFFFF {
            value - 0x8000
        } else {
            value
        };
        Ok(Co2Correction(value))
    }
}

impl From<Co2Correction> for u16 {
    fn from(value: Co2Correction) -> Self {
        value.0
    }
}

/// Ambient pressure value used for CO2 measurement compensation in hPa. Must be between 700hPa and
/// 1,200 hPa. The default value is 1,013 hPa.
#[derive(Debug, PartialEq)]
pub struct AmbientPressure(u16);

impl TryFrom<u16> for AmbientPressure {
    type Error = DataError;

    /// Create an [`AmbientPressure`] value for CO2 compensation. Value ranges are checked
    ///
    /// # Errors
    ///
    /// - [`ValueOutOfRange`](crate::error::DataError::ValueOutOfRange): If the ambient pressure is
    ///   not between 700 and 1,200 hPa.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        check_range(value, 700, 1_200, "Ambient Pressure", "hPa")?;
        Ok(AmbientPressure(value))
    }
}

impl TryFrom<&[u8]> for AmbientPressure {
    type Error = DataError;

    /// Parse the ambient pressure value from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 3)?;
        Ok(AmbientPressure(u16::from_be_bytes([data[0], data[1]])))
    }
}

impl From<AmbientPressure> for u16 {
    fn from(value: AmbientPressure) -> Self {
        value.0
    }
}

impl Default for AmbientPressure {
    /// Returns the default ambient pressure of 1,013 hPa.
    fn default() -> Self {
        Self(1013)
    }
}

/// Sensor altitude for CO2 measurement compensation in m above sea level. Must be between 0 m and
/// 3,000 m. The default value is 0 m.
#[derive(Debug, PartialEq)]
pub struct SensorAltitude(u16);

impl TryFrom<u16> for SensorAltitude {
    type Error = DataError;

    /// Create an [`SensorAltitude`] value for CO2 compensation. Value ranges are checked.
    ///
    /// # Errors
    ///
    /// - [`ValueOutOfRange`](crate::error::DataError::ValueOutOfRange): If the sensor altitude is
    ///   not between 0 and 3,000 m.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        check_range(value, 0, 3_000, "Sensor Altitude", "m")?;
        Ok(SensorAltitude(value))
    }
}

impl TryFrom<&[u8]> for SensorAltitude {
    type Error = DataError;

    /// Parse the sensor altitude from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 3)?;
        Ok(SensorAltitude(u16::from_be_bytes([data[0], data[1]])))
    }
}

impl From<SensorAltitude> for u16 {
    fn from(value: SensorAltitude) -> Self {
        value.0
    }
}

impl Default for SensorAltitude {
    /// Returns the default ambient pressure of 1,013 hPa.
    fn default() -> Self {
        Self(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_co2_concentration_wraps_raw_value() {
        let value = 12;
        assert_eq!(u16::from(TargetCO2Concentration::from(value)), value)
    }
}
