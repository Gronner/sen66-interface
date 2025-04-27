use crate::{error::DataError, util::check_deserialization};

/// One measurement taken from the SEN66. Use
/// [`read_measured_values`](crate::asynch::Sen66::read_measured_values) to retrieve it.
#[derive(Debug, PartialEq)]
pub struct Measurement {
    /// Mass concentration for PM1.0 in ug/m³.
    pub pm1_0: f32,
    /// Mass concentration for PM2.5 in ug/m³.
    pub pm2_5: f32,
    /// Mass concentration for PM4.0 in ug/m³.
    pub pm4_0: f32,
    /// Mass concentration for PM10.0 in ug/m³.
    pub pm10_0: f32,
    /// Relative Humidity in %.
    pub relative_humidity: f32,
    /// Temperature in °C.
    pub temperature: f32,
    /// VOC Index.
    pub voc_index: f32,
    /// NOx Index.
    pub nox_index: f32,
    /// CO2 concentration in ppm.
    pub co2: u16,
}

impl TryFrom<&[u8]> for Measurement {
    type Error = DataError;

    /// Parse the measurement from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 27)?;
        Ok(Self {
            pm1_0: u16::from_be_bytes([data[0], data[1]]) as f32 / 10.,
            pm2_5: u16::from_be_bytes([data[3], data[4]]) as f32 / 10.,
            pm4_0: u16::from_be_bytes([data[6], data[7]]) as f32 / 10.,
            pm10_0: u16::from_be_bytes([data[9], data[10]]) as f32 / 10.,
            relative_humidity: i16::from_be_bytes([data[12], data[13]]) as f32 / 100.,
            temperature: i16::from_be_bytes([data[15], data[16]]) as f32 / 200.,
            voc_index: i16::from_be_bytes([data[18], data[19]]) as f32 / 10.,
            nox_index: i16::from_be_bytes([data[21], data[22]]) as f32 / 10.,
            co2: u16::from_be_bytes([data[24], data[25]]),
        })
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Measurement {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "PM1.0:     {} ug/m³
PM2.5:     {} ug/m³
PM4.0:     {} ug/m³
PM10.0:    {} ug/m³
RH:        {} %
Temp:      {} °C
VOC Index: {} / 1
NOx Index: {} / 100
CO2:       {} ppm",
            self.pm1_0,
            self.pm2_5,
            self.pm4_0,
            self.pm10_0,
            self.relative_humidity,
            self.temperature,
            self.voc_index,
            self.nox_index,
            self.co2
        )
    }
}

/// One raw measurement taken from the SEN66. Use
/// [`read_measured_raw_values`](crate::asynch::Sen66::read_measured_raw_values) to retrieve it.
#[derive(Debug, PartialEq)]
pub struct RawMeasurement {
    /// Relative Humidity in %.
    pub relative_humidity: f32,
    /// Temperature in °C.
    pub temperature: f32,
    /// VOC ticks without scale facot
    pub voc: u16,
    /// NOx ticks without scale facot
    pub nox: u16,
    /// Uninterpolated CO2 concentration in ppm, updated every 5 seconds.
    pub co2: u16,
}

impl TryFrom<&[u8]> for RawMeasurement {
    type Error = DataError;

    /// Parse the raw measurement from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 15)?;
        Ok(Self {
            relative_humidity: i16::from_be_bytes([data[0], data[1]]) as f32 / 100.,
            temperature: i16::from_be_bytes([data[3], data[4]]) as f32 / 200.,
            voc: u16::from_be_bytes([data[6], data[7]]),
            nox: u16::from_be_bytes([data[9], data[10]]),
            co2: u16::from_be_bytes([data[12], data[13]]),
        })
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for RawMeasurement {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "RH:     {} %
Temp:   {} °C
VOC:    {} ticks
NOx:    {} ticks
CO2:    {} ppm",
            self.relative_humidity,
            self.temperature,
            self.voc,
            self.nox,
            self.co2
        )
    }
}

/// One concentration measurement taken from the SEN66. Use
/// [`read_number_concentrations`](crate::asynch::Sen66::read_number_concentrations) to retrieve it.
#[derive(Debug, PartialEq)]
pub struct Concentrations {
    /// PM0.5 concentration in particles/cm³
    pub pm0_5: f32,
    /// PM1.0 concentration in particles/cm³
    pub pm1_0: f32,
    /// PM2.5 concentration in particles/cm³
    pub pm2_5: f32,
    /// PM4.0 concentration in particles/cm³
    pub pm4_0: f32,
    /// PM10.0 concentration in particles/cm³
    pub pm10_0: f32,
}

impl TryFrom<&[u8]> for Concentrations {
    type Error = DataError;

    /// Parse the concentration  measurement from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 15)?;
        Ok(Self {
            pm0_5: u16::from_be_bytes([data[0], data[1]]) as f32 / 10.,
            pm1_0: u16::from_be_bytes([data[3], data[4]]) as f32 / 10.,
            pm2_5: u16::from_be_bytes([data[6], data[7]]) as f32 / 10.,
            pm4_0: u16::from_be_bytes([data[9], data[10]]) as f32 / 10.,
            pm10_0: u16::from_be_bytes([data[12], data[13]]) as f32 / 10.,
        })
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Concentrations {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "PM0.5:  {} p/cm³
PM1.0:  {} p/cm³
PM2.5:  {} p/cm³
PM4.0:  {} p/cm³
PM10.0: {} p/cm³",
            self.pm0_5,
            self.pm1_0,
            self.pm2_5,
            self.pm4_0,
            self.pm10_0
        )
    }
}
