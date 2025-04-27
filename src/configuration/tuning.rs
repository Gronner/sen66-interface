use crate::{
    error::DataError,
    util::{check_deserialization, check_range},
};

/// Configuration for the VOC Index algorithm.
#[derive(Debug, PartialEq)]
pub struct VocTuning(Tuning);

impl VocTuning {
    /// Creates a new [`VocTuning`](VocTuning) Index configuration:
    /// - `index_offset`: VOC Index representing typical conditions. Range: 1 - 250, Default: 100.
    /// - `learning_time_offset`: Time constant to estimate the offset from the history in hours.
    ///   After twice the learning time events are forgotten. Range: 1 - 1,000h, Default: 12h
    /// - `learning_time_gain`: Time constant to estimate the gain from the history in hours.
    ///   After twice the learning time events are forgotten. Range 1 - 1,000h, Default: 12h
    /// - `gating_max_durations`: Maximum duration the estimator freezes on a high VOC index
    ///   signal. Zero disables the gating. Range 0 - 3,000min, Default 180min.
    /// - `initial_standard_deviation`: Initial estimate for the standard deviation. Range 10 -
    ///   5,000, Default: 50.
    /// - `gain_factor`: Factor to amplify/attunate the VOC index output. Range 1 - 1,000, Default:
    ///   230.
    ///
    /// # Errors
    ///
    /// - [`ValueOutOfRange`](crate::error::DataError::ValueOutOfRange)`: If the values with scaling
    ///   are not in range.
    pub fn new(
        index_offset: i16,
        learning_time_offset: i16,
        learning_time_gain: i16,
        gating_max_durations: i16,
        initial_standard_deviation: i16,
        gain_factor: i16,
    ) -> Result<Self, DataError> {
        Ok(Self(Tuning::new(
            index_offset,
            learning_time_offset,
            learning_time_gain,
            gating_max_durations,
            initial_standard_deviation,
            gain_factor,
        )?))
    }
}

impl From<VocTuning> for [u16; 6] {
    fn from(value: VocTuning) -> Self {
        <[u16; 6]>::from(value.0)
    }
}

impl TryFrom<&[u8]> for VocTuning {
    type Error = DataError;

    /// Parse the VOC tuning parameters from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(VocTuning(Tuning::try_from(data)?))
    }
}

impl Default for VocTuning {
    /// Creates a default [`VocTuning`](VocTuning) with:
    /// - `index_offset`: 100
    /// - `learning_time_offset`: 12h
    /// - `learning_time_gain`: 12h
    /// - `gating_max_durations`: 180min
    /// - `initial_standard_deviation`: 50
    /// - `gain_factor`: 230
    fn default() -> Self {
        Self(Tuning {
            index_offset: 100,
            learning_time_offset: 12,
            learning_time_gain: 12,
            gating_max_durations: 180,
            initial_standard_deviation: 50,
            gain_factor: 230,
        })
    }
}

/// Configuration for the NOx Index algorithm.
#[derive(Debug, PartialEq)]
pub struct NoxTuning(Tuning);

impl NoxTuning {
    /// Creates a new [`NoxTuning`](NoxTuning) Index configuration:
    /// - `index_offset`: NOx Index representing typical conditions. Range 1 - 250, Default: 1.
    /// - `learning_time_offset`: Time constant to estimate the offset from the history in hours.
    ///   After twice the learning time events are forgotten. Range 1 - 1,000h, Default 12h.
    /// - `learning_time_gain`: Time constant to estimate the gain from the history in hours.
    ///   After twice the learning time events are forgotten. Range 1 - 1,000h, Default 12h.
    /// - `gating_max_durations`: Maximum duration the estimator freezes on a high NOx index
    ///   signal. Zero disables the gating. Range 0 - 3,000min, Default: 720min.
    /// - `gain_factor`: Factor to amplify/attunate the NOx index output. Range 1 - 1,000, Default:
    ///   230.
    ///
    /// # Errors
    ///
    /// - [`ValueOutOfRange`](crate::error::DataError::ValueOutOfRange)`: If the values with scaling
    ///   are not in range.
    pub fn new(
        index_offset: i16,
        learning_time_offset: i16,
        learning_time_gain: i16,
        gating_max_durations: i16,
        gain_factor: i16,
    ) -> Result<Self, DataError> {
        Ok(Self(Tuning::new(
            index_offset,
            learning_time_offset,
            learning_time_gain,
            gating_max_durations,
            50,
            gain_factor,
        )?))
    }
}

impl From<NoxTuning> for [u16; 6] {
    fn from(value: NoxTuning) -> Self {
        <[u16; 6]>::from(value.0)
    }
}

impl TryFrom<&[u8]> for NoxTuning {
    type Error = DataError;

    /// Parse the NOx tuning parameters from the received data.
    ///
    /// # Errors
    ///
    /// - [`CrcFailed`](crate::error::DataError::CrcFailed): If the received data CRC indicates
    ///   corruption.
    /// - [`ReceivedBufferWrongSize`](crate::error::DataError::ReceivedBufferWrongSize): If the
    ///   received data buffer is not the expected size.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(NoxTuning(Tuning::try_from(data)?))
    }
}

impl Default for NoxTuning {
    /// Creates a default [`NoxTuning`](NoxTuning) with:
    /// - `index_offset`: 1
    /// - `learning_time_offset`: 12h
    /// - `learning_time_gain`: 12h
    /// - `gating_max_durations`: 720min
    /// - `initial_standard_deviation`: 50
    /// - `gain_factor`: 230
    fn default() -> Self {
        Self(Tuning {
            index_offset: 100,
            learning_time_offset: 12,
            learning_time_gain: 12,
            gating_max_durations: 180,
            initial_standard_deviation: 50,
            gain_factor: 230,
        })
    }
}

#[derive(Debug, PartialEq)]
struct Tuning {
    index_offset: i16,
    learning_time_offset: i16,
    learning_time_gain: i16,
    gating_max_durations: i16,
    initial_standard_deviation: i16,
    gain_factor: i16,
}

impl Tuning {
    fn new(
        index_offset: i16,
        learning_time_offset: i16,
        learning_time_gain: i16,
        gating_max_durations: i16,
        initial_standard_deviation: i16,
        gain_factor: i16,
    ) -> Result<Self, DataError> {
        Ok(Self {
            index_offset: check_range(index_offset, 1, 250, "VOC Index Offset", "")?,
            learning_time_offset: check_range(
                learning_time_offset,
                1,
                1_000,
                "VOC Learning Time Offset",
                "h",
            )?,
            learning_time_gain: check_range(
                learning_time_gain,
                1,
                1_000,
                "VOC Learning Time Gain",
                "h",
            )?,
            gating_max_durations: check_range(
                gating_max_durations,
                0,
                3_000,
                "VOC Gating Max Duration",
                "min",
            )?,
            initial_standard_deviation: check_range(
                initial_standard_deviation,
                10,
                5_000,
                "VOC Initial Standard Deviation",
                "",
            )?,
            gain_factor: check_range(gain_factor, 1, 1_000, "VOC Gain Factor", "")?,
        })
    }
}

impl From<Tuning> for [u16; 6] {
    fn from(value: Tuning) -> Self {
        [
            value.index_offset as u16,
            value.learning_time_offset as u16,
            value.learning_time_gain as u16,
            value.gating_max_durations as u16,
            value.initial_standard_deviation as u16,
            value.gain_factor as u16,
        ]
    }
}

impl TryFrom<&[u8]> for Tuning {
    type Error = DataError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 18)?;
        Tuning::new(
            i16::from_be_bytes([data[0], data[1]]),
            i16::from_be_bytes([data[3], data[4]]),
            i16::from_be_bytes([data[6], data[7]]),
            i16::from_be_bytes([data[9], data[10]]),
            i16::from_be_bytes([data[12], data[13]]),
            i16::from_be_bytes([data[15], data[16]]),
        )
    }
}
