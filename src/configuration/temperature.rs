use crate::{error::DataError, util::check_scaling};

/// Temperature offset parameters to compensate temperature effects of the sensor's design-in
/// using:
/// `T_Ambient_Compensated = T_Ambient + (slope * T_Ambient) + offset`
/// Up to 5 temperature offsets can be stored.
pub struct TemperatureOffset {
    offset: i16,
    slope: i16,
    time_constant: u16,
    slot: u16,
}

impl TemperatureOffset {
    /// Creates a new [`TemperatureOffset`](TemperatureOffset) configuration with the given scaled parameters:
    /// - `offset`: Constant temperature offset in °C. Applied scale factor: 200
    /// - `slope`: Normalized temperature offset slope. Applied scale factor: 10,000
    /// - `time_constant`: Time constant determining how fast the new slope and offset are applied.
    /// - `slot`: Temperature offset slot to modify. Available slots range from 0 to 4.
    ///
    /// # Errors
    ///
    /// - [`ValueOutOfRange`](crate::error::DataError::ValueOutOfRange)`: If the values with scaling
    ///   are not in range.
    pub fn new(offset: i16, slope: i16, time_constant: u16, slot: u16) -> Result<Self, DataError> {
        Ok(Self {
            offset: check_scaling(offset, 200, "Temperature Offset", "°C")?,
            slope: check_scaling(slope, 10_000, "Temperature Slope", "")?,
            time_constant,
            slot: if (0..=4).contains(&slot) {
                slot
            } else {
                return Err(DataError::ValueOutOfRange {
                    parameter: "Temperature Slope",
                    min: 0,
                    max: 4,
                    unit: "",
                });
            },
        })
    }
}

impl From<TemperatureOffset> for [u16; 4] {
    fn from(value: TemperatureOffset) -> Self {
        [
            value.offset as u16,
            value.slope as u16,
            value.time_constant,
            value.slot,
        ]
    }
}

/// Temperature acceleration parameters for the RH/T engine. No documentation on these has been
/// published so far.
pub struct TemperatureAcceleration {
    k: u16,
    p: u16,
    t1: u16,
    t2: u16,
}

impl TemperatureAcceleration {
    /// Creates a new [`TemperatureAcceleration`](TemperatureAcceleration configuration with the given parameters. All
    /// parameters are scaled with a factor of 10.
    ///
    /// # Errors
    ///
    /// - [`ValueOutOfRange`](crate::error::DataError::ValueOutOfRange)`: If the values with scaling
    ///   are not in range.
    pub fn new(k: u16, p: u16, t1: u16, t2: u16) -> Result<Self, DataError> {
        Ok(Self {
            k: check_scaling(k, 10, "Temperature Acceleration K", "")?,
            p: check_scaling(p, 10, "Temperature Acceleration P", "")?,
            t1: check_scaling(t1, 10, "Temperature Acceleration T1", "")?,
            t2: check_scaling(t2, 10, "Temperature Acceleration T2", "")?,
        })
    }
}

impl From<TemperatureAcceleration> for [u16; 4] {
    fn from(value: TemperatureAcceleration) -> Self {
        [value.k, value.p, value.t1, value.t2]
    }
}
