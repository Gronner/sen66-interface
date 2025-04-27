use crate::{error::DataError, util::check_deserialization};

/// Name of the sensor in ASCII
#[derive(Clone, Copy)]
pub struct ProductName(SmallString);

impl TryFrom<&[u8]> for ProductName {
    type Error = DataError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(ProductName(value.try_into()?))
    }
}

impl ProductName {
    /// Provides access the underlying buffer
    pub fn get_name_buffer(&self) -> &[u8] {
        self.0.get_buffer()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ProductName {
    /// Writes the defmt representation to the Formatter.
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", self.0)
    }
}

/// Name of the sensor in ASCII
#[derive(Clone, Copy)]
pub struct SerialNumber(SmallString);

impl SerialNumber {
    /// Provides access the underlying buffer
    pub fn get_serial_buffer(&self) -> &[u8] {
        self.0.get_buffer()
    }
}

impl TryFrom<&[u8]> for SerialNumber {
    type Error = DataError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(SerialNumber(value.try_into()?))
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SerialNumber {
    /// Writes the defmt representation to the Formatter.
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy)]
struct SmallString {
    name: [u8; 32],
    len: usize,
}

impl SmallString {
    fn get_buffer(&self) -> &[u8] {
        &self.name[0..self.len]
    }
}

impl TryFrom<&[u8]> for SmallString {
    type Error = DataError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        check_deserialization(data, 48)?;
        let mut name = [0; 32];
        let mut len = 0;
        for (i, &c) in data.iter().enumerate() {
            // Skip CRC bytes
            if (i + 1) % 3 == 0 {
                continue;
            }
            if !c.is_ascii() {
                return Err(Self::Error::NotASCIIString);
            }
            name[len] = c;
            len += 1;
            if c == 0x00 {
                break;
            }
        }

        Ok(Self { name, len })
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SmallString {
    /// Writes the defmt representation to the Formatter.
    fn format(&self, f: defmt::Formatter) {
        // ProductName::TryFrom ensures that the data contained is ASCII and not longer than the
        // first null-terminator
        let output = unsafe { core::str::from_utf8_unchecked(&self.name[0..self.len]) };
        defmt::write!(f, "{}", output)
    }
}
