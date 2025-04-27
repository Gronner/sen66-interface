use crate::{error::DataError, util::check_deserialization};

const DATA_STATUS_VALUE: &str = "Data ready status";
const DATA_STATUS_EXPECTED: &str = "0 or 1";

/// Describes whether a new measurement is ready to be read from the sensor.
#[derive(Debug, PartialEq)]
pub enum DataStatus {
    /// New Data is ready and can be read.
    Ready,
    /// No new data is available to be read.
    NotReady,
}

#[cfg(feature = "defmt")]
impl defmt::Format for DataStatus {
    fn format(&self, f: defmt::Formatter) {
        match self {
            DataStatus::Ready => defmt::write!(f, "Ready"),
            DataStatus::NotReady => defmt::write!(f, "Not Ready"),
        }
    }
}

impl TryFrom<&[u8]> for DataStatus {
    type Error = DataError;

    /// Converts buffered data to an [DataStatus] value.
    ///
    /// # Errors
    ///
    /// - [ReceivedBufferWrongSize](crate::error::DataError::ReceivedBufferWrongSize) if the `data`
    ///   buffer is not big enough for the data that should have been received.
    /// - [CrcFailed](crate::error::DataError::CrcFailed) if the CRC of the received data does not
    ///   match.
    /// - [UnexpectedValueReceived](crate::error::DataError::UnexpectedValueReceived) if the
    ///   received value is not `0` or `1`.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 3)?;
        match data[1] {
            0x00 => Ok(Self::NotReady),
            0x01 => Ok(Self::Ready),
            val => Err(DataError::UnexpectedValueReceived {
                parameter: DATA_STATUS_VALUE,
                expected: DATA_STATUS_EXPECTED,
                actual: val as u16,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_status_is_enabled_if_one_is_send() {
        let data = [0x00, 0x01, 0xB0];
        assert_eq!(DataStatus::try_from(&data[..]).unwrap(), DataStatus::Ready);
    }

    #[test]
    fn data_status_is_disabled_if_zero_is_send() {
        let data = [0x00, 0x00, 0x81];
        assert_eq!(
            DataStatus::try_from(&data[..]).unwrap(),
            DataStatus::NotReady
        );
    }

    #[test]
    fn receiving_invalid_value_for_data_status_emits_error() {
        let data = [0x00, 0x03, 0xD2];
        assert!(DataStatus::try_from(&data[..]).is_err());
    }
}
