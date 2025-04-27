use crate::{
    error::{DataError, DeviceError},
    util::{check_deserialization, is_set},
};

/// Represents the state of the sensor.
#[derive(Debug, PartialEq)]
pub enum SensorState {
    /// Sensor is in idle state. Either after power-on, a reset or when calling
    /// [`stop_measurement`](crate::asynch::Sen66::stop_measurement).
    Idle,
    /// Sensor is in measuring state. Entered by calling
    /// [`start_measurement`](crate::asynch::Sen66::start_measurement).
    Measuring,
}

#[cfg(feature = "defmt")]
impl defmt::Format for SensorState {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", self)
    }
}

/// Sensor status register.
#[derive(Debug, PartialEq)]
pub struct DeviceStatusRegister(u32);

impl DeviceStatusRegister {
    /// Returns whether a fan speed warning is present, as the speed is off more than 10% for
    /// multiple measurement intervals. Disappears if the issue disappears.
    pub fn fan_speed_warning(&self) -> bool {
        is_set(self.0, 21)
    }

    /// Returns whether the PM sensor exhibits an error.
    /// <div class="warning">Persists even if the error disappears. Requires reseting the devices
    /// status, the device or performing a power cycle.</div>
    pub fn pm_sensor_error(&self) -> bool {
        is_set(self.0, 11)
    }

    /// Returns whether the CO2 sensor exhibits an error.
    /// <div class="warning">Persists even if the error disappears. Requires reseting the devices
    /// status, the device or performing a power cycle.</div>
    pub fn co2_sensor_error(&self) -> bool {
        is_set(self.0, 9)
    }

    /// Returns whether the Gas sensor exhibits an error.
    /// <div class="warning">Persists even if the error disappears. Requires reseting the devices
    /// status, the device or performing a power cycle.</div>
    pub fn gas_sensor_error(&self) -> bool {
        is_set(self.0, 7)
    }

    /// Returns whether the RH/T sensor exhibits an error.
    /// <div class="warning">Persists even if the error disappears. Requires reseting the devices
    /// status, the device or performing a power cycle.</div>
    pub fn rht_sensor_error(&self) -> bool {
        is_set(self.0, 6)
    }

    /// Returns whether the fan exhibits an error: It is turned on, but 0RPM are reported over
    /// multiple measurement intervals.
    /// <div class="warning">Persists even if the error disappears. Requires reseting the devices
    /// status, the device or performing a power cycle.</div>
    pub fn fan_error(&self) -> bool {
        is_set(self.0, 4)
    }

    /// Checks whether any error has occured
    ///
    /// # Errors
    ///
    /// - [`DeviceError`](crate::error::DeviceError): Returned when any error is present, flags
    ///   indicate which errors are present.
    pub fn has_error(&self) -> Result<(), DeviceError> {
        let pm = self.pm_sensor_error();
        let co2 = self.co2_sensor_error();
        let gas = self.gas_sensor_error();
        let rht = self.rht_sensor_error();
        let fan = self.fan_error();
        if [pm, co2, gas, rht, fan].iter().any(|&err| err) {
            Err(DeviceError {
                pm,
                co2,
                gas,
                rht,
                fan,
            })
        } else {
            Ok(())
        }
    }
}

impl TryFrom<&[u8]> for DeviceStatusRegister {
    type Error = DataError;

    /// Parse the device status register from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 6)?;
        Ok(DeviceStatusRegister(u32::from_be_bytes([
            data[0], data[1], data[3], data[4],
        ])))
    }
}

/// Indicates whether automatic self calibration (ASC) is enabled.
#[derive(Debug, PartialEq)]
pub enum AscState {
    /// ASC is enabled.
    Enabled,
    /// ASC is disabled.
    Disabled,
}

impl TryFrom<&[u8]> for AscState {
    type Error = DataError;

    /// Parse the ASC state from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    /// - [UnexpectedValueReceived](crate::error::DataError::UnexpectedValueReceived) if the
    ///   received value is not `0` or `1`.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 3)?;
        match data[1] {
            0x00 => Ok(Self::Disabled),
            0x01 => Ok(Self::Enabled),
            val => Err(DataError::UnexpectedValueReceived {
                parameter: "ASC State",
                expected: "0 or 1",
                actual: val as u16,
            }),
        }
    }
}

impl From<AscState> for u16 {
    fn from(value: AscState) -> Self {
        match value {
            AscState::Enabled => 0x0001,
            AscState::Disabled => 0x0000,
        }
    }
}

/// Stores the VOC algorithm state, which can be used to skip the learning phase after a power
/// cycle.
#[derive(Debug, PartialEq)]
pub struct VocAlgorithmState([u8; 8]);

impl TryFrom<&[u8]> for VocAlgorithmState {
    type Error = DataError;

    /// Parse the VOC algorithm state from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 12)?;
        Ok(VocAlgorithmState([
            data[0], data[1], data[3], data[4], data[6], data[7], data[9], data[10],
        ]))
    }
}

impl From<VocAlgorithmState> for [u16; 4] {
    fn from(value: VocAlgorithmState) -> Self {
        [
            u16::from_be_bytes([value.0[0], value.0[1]]),
            u16::from_be_bytes([value.0[2], value.0[3]]),
            u16::from_be_bytes([value.0[4], value.0[5]]),
            u16::from_be_bytes([value.0[6], value.0[7]]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_flags_set_nothing_reported() {
        let state = DeviceStatusRegister(0b0000_0000_0000_0000_0000_0000_0000_0000);
        assert!(!state.fan_speed_warning());
        assert!(state.has_error().is_ok());
    }

    #[test]
    fn set_fan_speed_warning_reported() {
        let state = DeviceStatusRegister(0b0000_0000_0010_0000_0000_0000_0000_0000);
        assert!(state.fan_speed_warning());
    }

    #[test]
    fn set_fan_speed_error_reported() {
        let state = DeviceStatusRegister(0b0000_0000_0000_0000_0000_0000_0001_0000);
        assert!(state.fan_error());
    }

    #[test]
    fn set_rht_error_reported() {
        let state = DeviceStatusRegister(0b0000_0000_0000_0000_0000_0000_0100_0000);
        assert!(state.rht_sensor_error());
    }

    #[test]
    fn set_gas_error_reported() {
        let state = DeviceStatusRegister(0b0000_0000_0000_0000_0000_0000_1000_0000);
        assert!(state.gas_sensor_error());
    }

    #[test]
    fn set_co2_error_reported() {
        let state = DeviceStatusRegister(0b0000_0000_0000_0000_0000_0010_0000_0000);
        assert!(state.co2_sensor_error());
    }

    #[test]
    fn set_pm_error_reported() {
        let state = DeviceStatusRegister(0b0000_0000_0000_0000_0000_1000_0000_0000);
        assert!(state.pm_sensor_error());
    }

    #[test]
    fn set_warning_flag_does_not_emit_error() {
        let state = DeviceStatusRegister(0b0000_0000_0010_0000_0000_0000_0000_0000);
        assert!(state.has_error().is_ok());
    }

    #[test]
    fn set_error_flag_does_emit_device_error() {
        let state = DeviceStatusRegister(0b0000_0000_0000_0000_0000_1000_0000_0000);
        assert_eq!(
            state.has_error().unwrap_err(),
            DeviceError {
                pm: true,
                co2: false,
                gas: false,
                rht: false,
                fan: false
            }
        );
    }

    #[test]
    fn deserialize_device_status_register_with_all_flags_set_yields_u32_with_flag_bits_one() {
        let data = [0x00, 0x20, 0x07, 0x0E, 0xD0, 0xE8];
        assert_eq!(
            DeviceStatusRegister::try_from(&data[..]).unwrap(),
            DeviceStatusRegister(0b0000_0000_0010_0000_0000_1110_1101_0000)
        );
    }

    #[test]
    fn deserialize_asc_status_enabled_yields_enabled() {
        let data = [0x00, 0x01, 0xB0];
        assert_eq!(AscState::try_from(&data[..]).unwrap(), AscState::Enabled);
    }

    #[test]
    fn deserialize_asc_status_disabled_yields_enabled() {
        let data = [0x00, 0x00, 0x81];
        assert_eq!(AscState::try_from(&data[..]).unwrap(), AscState::Disabled);
    }

    #[test]
    fn deserialize_asc_status_unknown_emit_error() {
        let data = [0x00, 0x03, 0xd2];
        assert!(AscState::try_from(&data[..]).is_err());
    }

    #[test]
    fn serialize_asc_status_enabled_yields_one() {
        assert_eq!(u16::from(AscState::Enabled), 0x0001);
    }

    #[test]
    fn serialize_asc_status_disabled_yields_zero() {
        assert_eq!(u16::from(AscState::Disabled), 0x0000);
    }

    #[test]
    fn deserialize_voc_algorithm_state_yields_same_state() {
        let data = [
            0x01, 0x02, 0x17, 0x03, 0x04, 0x68, 0x05, 0x06, 0x50, 0x07, 0x08, 0x96,
        ];
        assert_eq!(
            VocAlgorithmState::try_from(&data[..]).unwrap(),
            VocAlgorithmState([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])
        );
    }

    #[test]
    fn serialize_voc_algorithm_state_yields_same_state() {
        assert_eq!(
            <[u16; 4]>::from(VocAlgorithmState([
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08
            ])),
            [0x0102, 0x0304, 0x0506, 0x0708]
        );
    }
}
