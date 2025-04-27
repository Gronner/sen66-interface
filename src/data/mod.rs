//! Data types for configuring the SEN66's operations.

mod data_status;
mod measurement;
mod product_data;
mod state;

pub use data_status::DataStatus;
pub use measurement::{Concentrations, Measurement, RawMeasurement};
pub use product_data::{ProductName, SerialNumber};
pub use state::{AscState, DeviceStatusRegister, SensorState, VocAlgorithmState};
