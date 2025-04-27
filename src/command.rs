//! SEN66 I2C Commands.

/// I2C Commands for the SEN66 according to its [interface
/// description](https://sensirion.com/media/documents/FAFC548D/6731FFFA/Sensirion_Datasheet_SEN6x.pdf).
#[derive(Clone, Copy)]
pub enum Command {
    /// Starts a continuous measurement and moves chip to measuring state. After the sending the command
    /// it might take some time until the first measurement is ready.
    /// Exec. Time: 50ms
    /// <div class="warning">Only available in idle state</div>
    StartContinuousMeasurement = 0x0021,
    /// Stops measurements and returns to idle state. Wait at least 1000ms until starting a new
    /// measurement.
    /// Exec. Time: 1000ms
    /// <div class="warning">Only available in measuring state</div>
    StopMeasurement = 0x0104,
    /// Queries whether a measurement can be read from the sensor's buffer. The answer is `1` if a
    /// measurement is available `0` otherwise.
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in measuring state</div>
    GetDataReady = 0x0202,
    /// If a measurement is available reads out the measurement. If no new data is available the
    /// previous measurement is returned. If no data is available all data is set to the maximum
    /// value (`0xFFFF` for `u16`, `0x7FFF` for `i16`). The measurement contains the mass
    /// concentration for PM1.0, PM2.5, PM4.0 and PM10.0 in ug/m³, the relative humidity in %, the temperature
    /// in °C, the [volatile organic compounds (VOC)
    /// index](https://sensirion.com/media/documents/02232963/6294E043/Info_Note_VOC_Index.pdf),
    /// the [NOx
    /// index](https://sensirion.com/media/documents/9F289B95/6294DFFC/Info_Note_NOx_Index.pdf) and
    /// CO2 concentration in ppm.
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in measuring state</div>
    ReadMeasurement = 0x0300,
    /// If a measurement is available reads out the measured raw values. If no new data is available
    /// the previous measurement is returned. If no data is available all data is set to the maximum
    /// value (`0xFFFF` for `u16`, `0x7FFF` for `i16`). The measurement contains the raw relative
    /// humidity in %, the raw temperature in °C, the VOC ticks, the NOx ticks and the CO2
    /// concentration in ppm. For the first 10-11s after power-on or device reset the CO2 value will
    /// be `0xFFFF`.
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in measuring state</div>
    ReadRawMeasurement = 0x0405,
    /// If a measurement is available reads out the measured number concentration values. If no
    /// new data is available the previous values will be returned. If no data is available at all,
    /// the data is set to the maximum value (`0xFFFF` for `u16`). The values contain the mass
    /// concentration for PM0.5, PM1.0, PM2.5, PM4.0 and PM10.0 in p/cm³
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in measuring state</div>
    ReadNumberConcentrationValues = 0x0316,
    /// Configures the temperature compensation via a slope and one of five offsets in °C.
    /// Exec. Time: 20ms
    SetTemperatureOffsetParameters = 0x60B2,
    /// Configures the temperature acceleration parameters for the RH/T engine. Thes parameters are
    /// volatile and reverted after a device reset.
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in idle state</div>
    SetTemperatureAccelerationParameters = 0x6100,
    /// Reads out the product name as a null-terminated ASCII string with up to 32 characters.
    /// Exec. Time: 20ms
    GetProductName = 0xD014,
    /// Reads out the device's serial number as a null-terminated ASCII string with up to 32
    /// characters.
    /// Exec. Time: 20ms
    GetSerialNumber = 0xD033,
    /// Read out the device's status register as a 32-bit bitfield.
    /// Exec. Time: 20ms
    GetDeviceStatus = 0xD206,
    /// Read the current device status as a 32-bit bitfield and clear all flags.
    /// Exec. Time: 20ms
    ReadAndClearDeviceStatus = 0xD210,
    /// Executes a device reset, the same as a power cycle.
    /// Exec. Time: 1200ms
    ResetDevice = 0xD304,
    /// Starts fan cleaning, where fan speed is set to a maximum for 10s. Wait at least 10s after
    /// this command until the next measurement.
    /// Exec. Time: 1ms
    /// <div class="warning">Only available in idle state</div>
    StartFanCleaning = 0x5607,
    /// Start the SHT's inbuilt heater for 1s with 200mW. Wait at least 20s after this command
    /// until the next measurement.
    /// Exec. Time: 1300ms
    /// <div class="warning">Only available in idle state</div>
    ActivateShtHeater = 0x3730,
    /// Sets or reads the parameters that customize the VOC algorithm. Contains the index offset,
    /// the learning time offset hours, the learning time gain hours, the max duration minutes, the
    /// initial standard deviation and the gain factor (all `i16`).
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in idle state</div>
    SetReadVocTuningParameters = 0x60D0,
    /// Sets or reads the state of the VOC algorithm to skip the initial learning phase. The state
    /// is encoded in a byte array of length 8.
    /// Exec. Time: 20ms
    /// <div class="warning">Writing only available in idle state</div>
    SetReadVocAlgorithmState = 0x6181,
    /// Sets or reads the parameters that customize the VOC algorithm. Contains the index offset,
    /// the learning time offset hours, the learning time gain hours, the max duration minutes, the
    /// initial standard deviation and the gain factor (all `i16`).
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in idle state</div>
    SetReadNoxTuningParameters = 0x60E1,
    /// Executes a forced recalibration (FRC) of the CO2 signal. Send the target CO2 concentation
    /// (as `u16`) and receive the correction factor as FRC - 0x8000 (as `u16`). Wait at least 1000ms after power-on
    /// and 600ms after stopping measurement to send this command.
    /// # Errors
    /// If recalibration failes 0xFFFF is returned.
    /// Exec. Time: 500ms
    /// <div class="warning">Only available in idle state</div>
    ForcedRecalibration = 0x6707,
    /// Enables/Disables or reads the status of the automatic self calibration (ASC) for the CO2
    /// sensor via a `bool` value. Sending a `0x01` activates ASC, sending a `0x00` disables ASC.
    /// Receiving a `0x01` indicates that ASC is enabled, a `0x00` indicates that ASC is disabled.
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in idle state</div>
    SetReadCo2AutomaticSelfCalibration = 0x6711,
    /// Sets or reads the ambient pressure value in hPA (as `u16`) which is used for the CO2
    /// sensor's pressure compensation.
    /// Exec. Time: 20ms
    SetReadAmbientPreassure = 0x6720,
    /// Sets or reads the sensors current altitude in m (as `u16`) which is used for the CO2
    /// sensor's pressure compensation.
    /// Exec. Time: 20ms
    /// <div class="warning">Only available in idle state</div>
    SetReadSensorAltitude = 0x6736,
}

impl Command {
    /// Returns a big endian byte representation of the command.
    pub const fn to_be_bytes(&self) -> [u8; 2] {
        (*self as u16).to_be_bytes()
    }

    /// Returns the execution_time of the command in ms.
    pub(crate) const fn execution_time_ms(&self) -> u32 {
        match self {
            Command::StartContinuousMeasurement => 50,
            Command::StopMeasurement => 1000,
            Command::GetDataReady => 20,
            Command::ReadMeasurement => 20,
            Command::ReadRawMeasurement => 20,
            Command::ReadNumberConcentrationValues => 20,
            Command::SetTemperatureOffsetParameters => 20,
            Command::SetTemperatureAccelerationParameters => 20,
            Command::GetProductName => 20,
            Command::GetSerialNumber => 20,
            Command::GetDeviceStatus => 20,
            Command::ReadAndClearDeviceStatus => 20,
            Command::ResetDevice => 20,
            Command::StartFanCleaning => 20,
            Command::ActivateShtHeater => 1300,
            Command::SetReadVocTuningParameters => 20,
            Command::SetReadVocAlgorithmState => 20,
            Command::SetReadNoxTuningParameters => 20,
            Command::ForcedRecalibration => 500,
            Command::SetReadCo2AutomaticSelfCalibration => 20,
            Command::SetReadAmbientPreassure => 20,
            Command::SetReadSensorAltitude => 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_to_bytes_works() {
        use Command::*;
        let data = [
            (StartContinuousMeasurement, [0x00, 0x21]),
            (StopMeasurement, [0x01, 0x04]),
            (GetDataReady, [0x02, 0x02]),
            (ReadMeasurement, [0x03, 0x00]),
            (ReadRawMeasurement, [0x04, 0x05]),
            (ReadNumberConcentrationValues, [0x03, 0x16]),
            (SetTemperatureOffsetParameters, [0x60, 0xB2]),
            (SetTemperatureAccelerationParameters, [0x61, 0x00]),
            (GetProductName, [0xD0, 0x14]),
            (GetSerialNumber, [0xD0, 0x33]),
            (GetDeviceStatus, [0xD2, 0x06]),
            (ReadAndClearDeviceStatus, [0xD2, 0x10]),
            (ResetDevice, [0xD3, 0x04]),
            (StartFanCleaning, [0x56, 0x07]),
            (ActivateShtHeater, [0x37, 0x30]),
            (SetReadVocTuningParameters, [0x60, 0xD0]),
            (SetReadVocAlgorithmState, [0x61, 0x81]),
            (SetReadNoxTuningParameters, [0x60, 0xE1]),
            (ForcedRecalibration, [0x67, 0x07]),
            (SetReadCo2AutomaticSelfCalibration, [0x67, 0x11]),
            (SetReadAmbientPreassure, [0x67, 0x20]),
            (SetReadSensorAltitude, [0x67, 0x36]),
        ];
        for (command, result) in data {
            assert_eq!(command.to_be_bytes(), result);
        }
    }
}
