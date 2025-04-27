use crate::error::DataError;

pub(crate) fn crc8_matches(data: &[u8], crc: u8) -> bool {
    compute_crc8(data) == crc
}

/// Computes the CRC-8-Dallas/Maxim (NRSC-5) for the provided data.
/// width=8 poly=0x31 init=0xff refin=false refout=false xorout=0x00 check=0xf7 residue=0x00 name="CRC-8/NRSC-5"
pub(crate) fn compute_crc8(data: &[u8]) -> u8 {
    const INITIAL: u8 = 0xFF;
    const POLYNOMIAL: u8 = 0x31;
    let mut crc = INITIAL;
    for byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ POLYNOMIAL;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

pub(crate) fn check_deserialization(data: &[u8], expected_len: usize) -> Result<(), DataError> {
    if data.len() != expected_len {
        return Err(DataError::ReceivedBufferWrongSize);
    }
    if data
        .chunks(3)
        .any(|chunk| !crc8_matches(&chunk[..2], chunk[2]))
    {
        return Err(DataError::CrcFailed);
    }
    Ok(())
}

pub(crate) fn check_scaling<T>(
    value: T,
    scalar: T,
    name: &'static str,
    unit: &'static str,
) -> Result<T, DataError>
where
    T: num::CheckedMul + num::Bounded + core::ops::Div<Output = T>,
    i32: From<T>,
{
    if let Some(value) = value.checked_mul(&scalar) {
        Ok(value)
    } else {
        Err(DataError::ValueOutOfRange {
            parameter: name,
            min: 0,
            max: (T::max_value() / scalar).into(),
            unit,
        })
    }
}

pub(crate) fn check_range<T>(
    value: T,
    min: T,
    max: T,
    name: &'static str,
    unit: &'static str,
) -> Result<T, DataError>
where
    T: Copy + PartialOrd,
    i32: From<T>,
{
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(DataError::ValueOutOfRange {
            parameter: name,
            min: min.into(),
            max: max.into(),
            unit,
        })
    }
}

#[inline]
pub(crate) const fn is_set(value: u32, bit: u32) -> bool {
    value & (1 << bit) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_crc_computes_properly() {
        let data = 0xBEEF_u16.to_be_bytes();
        let result = compute_crc8(&data);
        assert_eq!(result, 0x92);
    }
}
