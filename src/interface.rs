use duplicate::duplicate_item;

const ADDRESS: u8 = 0x6B;
const WRITE_FLAG: u8 = 0x00;
const READ_FLAG: u8 = 0x01;

// `await` replacement needs to be a callable due to the dot notation. This tricks enables that
// use case.
#[cfg(not(tarpaulin_include))]
trait Identity: Sized {
    fn identity(self) -> Self {
        core::convert::identity(self)
    }
}

impl<T: Sized> Identity for T {}

#[duplicate_item(
    feature_        module      async   await               delay_trait                             i2c_trait                                       test_macro;
    ["async"]       [asynch]    [async] [await.identity()]  [embedded_hal_async::delay::DelayNs]    [embedded_hal_async::i2c::I2c<Error = ERR>]  [tokio::test];
    ["blocking"]    [blocking]  []      [identity()]        [embedded_hal::delay::DelayNs]          [embedded_hal::i2c::I2c<Error = ERR>]        [test];
)]
pub mod module {
    //! Implementation of the SCD30's interface
    #[cfg(feature=feature_)]
    mod inner {
        use crate::{
            command::Command,
            configuration::{
                AmbientPressure, Co2Correction, NoxTuning, SensorAltitude, TargetCO2Concentration,
                TemperatureAcceleration, TemperatureOffset, VocTuning,
            },
            data::{
                AscState, Concentrations, DataStatus, DeviceStatusRegister, Measurement,
                ProductName, RawMeasurement, SensorState, SerialNumber, VocAlgorithmState,
            },
            error::Sen66Error,
            interface::{ADDRESS, Identity, READ_FLAG, WRITE_FLAG},
            util::compute_crc8,
        };

        /// Interface for the SEN66.
        pub struct Sen66<DELAY, I2C> {
            delay: DELAY,
            i2c: I2C,
            state: SensorState,
        }

        impl<DELAY: delay_trait, I2C: i2c_trait, ERR: embedded_hal::i2c::Error> Sen66<DELAY, I2C> {
            /// Creates a new SEN66 interface.
            /// - `delay`: Delay provider, implementing embedded_hal's `DelayNs` trait.
            /// - `i2c`: I2C peripheral implementing embedded_hal's `I2c` trait.
            pub fn new(delay: DELAY, i2c: I2C) -> Self {
                Self {
                    delay,
                    i2c,
                    state: SensorState::Idle,
                }
            }

            /// Starts a continous measurement. The first result is available after roughly 1.1s
            /// use [`is_data_ready`](Sen66::is_data_ready) to poll for available measurements.
            /// Changes sensors state to [`Measuring`](crate::data::SensorState).
            /// Execution Time: 50ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Measuring state.
            pub async fn start_measurement(&mut self) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<2>(Command::StartContinuousMeasurement, None)
                    .await?;
                self.state = SensorState::Measuring;
                Ok(())
            }

            /// Stops continous measurements.
            /// Changes sensors state to [`Idle`](crate::data::SensorState).
            /// Execution Time: 1000ms
            /// <div class="warning">Only available in measuring state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            pub async fn stop_measurement(&mut self) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Measuring {
                    return Err(Sen66Error::WrongState("Idle"));
                }
                self.write::<2>(Command::StopMeasurement, None).await?;
                self.state = SensorState::Idle;
                Ok(())
            }

            /// Queries whether new data is available.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in measuring state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn is_data_ready(&mut self) -> Result<DataStatus, Sen66Error<ERR>> {
                if self.state != SensorState::Measuring {
                    return Err(Sen66Error::WrongState("Idle"));
                }
                let received = self.write_read::<2, 3>(Command::GetDataReady, None).await?;
                Ok(DataStatus::try_from(&received[..])?)
            }

            /// Read a [`Measurement`](crate::data::Measurement) value from the sensor.
            /// If new data is available clears the data ready flag. If no new data is available
            /// the previous data point is returned. If no data at all is available all values are
            /// set to their maximum value.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in measuring state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn read_measured_values(&mut self) -> Result<Measurement, Sen66Error<ERR>> {
                if self.state != SensorState::Measuring {
                    return Err(Sen66Error::WrongState("Idle"));
                }
                let received = self
                    .write_read::<2, 27>(Command::ReadMeasurement, None)
                    .await?;
                Ok(Measurement::try_from(&received[..])?)
            }

            /// Read a [`RawMeasurement`](crate::data::RawMeasurement) value from the sensor.
            /// If new data is available clears the data ready flag. If no new data is available
            /// the previous data point is returned. If no data at all is available all values are
            /// set to their maximum value.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in measuring state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn read_measured_raw_values(
                &mut self,
            ) -> Result<RawMeasurement, Sen66Error<ERR>> {
                if self.state != SensorState::Measuring {
                    return Err(Sen66Error::WrongState("Idle"));
                }
                let received = self
                    .write_read::<2, 15>(Command::ReadRawMeasurement, None)
                    .await?;
                Ok(RawMeasurement::try_from(&received[..])?)
            }

            /// Read a [`Concentrations`](crate::data::Concentrations) value from the sensor.
            /// If new data is available clears the data ready flag. If no new data is available
            /// the previous data point is returned. If no data at all is available all values are
            /// set to their maximum value.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in measuring state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn read_number_concentrations(
                &mut self,
            ) -> Result<Concentrations, Sen66Error<ERR>> {
                if self.state != SensorState::Measuring {
                    return Err(Sen66Error::WrongState("Idle"));
                }
                let received = self
                    .write_read::<2, 15>(Command::ReadNumberConcentrationValues, None)
                    .await?;
                Ok(Concentrations::try_from(&received[..])?)
            }

            /// Set the temperature offset parameters.
            /// - `parameter`: See [`TemperatureOffset`](crate::configuration::TemperatureOffset)
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            pub async fn set_temperature_offset(
                &mut self,
                parameter: TemperatureOffset,
            ) -> Result<(), Sen66Error<ERR>> {
                Ok(self
                    .write::<14>(
                        Command::SetTemperatureOffsetParameters,
                        Some(&(<[u16; 4]>::from(parameter))),
                    )
                    .await?)
            }

            /// Set the temperature acceleration parameters.
            /// - `parameter`: See [`TemperatureAcceleration`](crate::configuration::TemperatureAcceleration)
            /// Execution Time: 20ms
            /// <div class="warning">Only available in Idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Measuring state.
            pub async fn set_temperature_acceleration(
                &mut self,
                parameter: TemperatureAcceleration,
            ) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                Ok(self
                    .write::<14>(
                        Command::SetTemperatureAccelerationParameters,
                        Some(&(<[u16; 4]>::from(parameter))),
                    )
                    .await?)
            }

            /// Read out the sensor's product name
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_product_name(&mut self) -> Result<ProductName, Sen66Error<ERR>> {
                let received = self
                    .write_read::<2, 48>(Command::GetProductName, None)
                    .await?;
                Ok(ProductName::try_from(&received[..])?)
            }

            /// Read out the sensor's serial number
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_serial_number(&mut self) -> Result<SerialNumber, Sen66Error<ERR>> {
                let received = self
                    .write_read::<2, 48>(Command::GetSerialNumber, None)
                    .await?;
                Ok(SerialNumber::try_from(&received[..])?)
            }

            /// Read out the sensor's [`DeviceStatusRegister`](crate::data::DeviceStatusRegister).
            /// Error flags are untouched by this.
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn read_device_status(
                &mut self,
            ) -> Result<DeviceStatusRegister, Sen66Error<ERR>> {
                let received = self
                    .write_read::<2, 6>(Command::GetDeviceStatus, None)
                    .await?;
                Ok(DeviceStatusRegister::try_from(&received[..])?)
            }

            /// Read out the sensor's [`DeviceStatusRegister`](crate::data::DeviceStatusRegister) and
            /// reset flags stored within.
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn read_and_clear_device_status(
                &mut self,
            ) -> Result<DeviceStatusRegister, Sen66Error<ERR>> {
                let received = self
                    .write_read::<2, 6>(Command::ReadAndClearDeviceStatus, None)
                    .await?;
                Ok(DeviceStatusRegister::try_from(&received[..])?)
            }

            /// Reset the sensor, akin to a power cycle.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Measuring state.
            pub async fn reset_device(&mut self) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<2>(Command::ResetDevice, None).await
            }

            /// Start the fan cleaning procedure.
            /// The fan is set to maximum speed for 10s and then stopped. After issuing this
            /// command wait at least 10s before starting a measurement.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Measuring state.
            pub async fn start_fan_cleaning(&mut self) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<2>(Command::StartFanCleaning, None).await
            }

            /// Activate the SHT heater.
            /// The heater runs with 200mW for 1s. Wait at least 20s after the command for the heat
            /// to disapper, before taking the next measurement.
            /// Execution Time: 1300ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Measuring state.
            pub async fn activate_sht_heater(&mut self) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<2>(Command::ActivateShtHeater, None).await
            }

            /// Read the [`VocTuning`](crate::configuration::VocTuning) parameters from the sensor.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_voc_tuning_parameters(
                &mut self,
            ) -> Result<VocTuning, Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                let received = self
                    .write_read::<2, 18>(Command::SetReadVocTuningParameters, None)
                    .await?;
                Ok(VocTuning::try_from(&received[..])?)
            }

            /// Set the [`VocTuning`](crate::configuration::VocTuning) parameters for the sensor.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            pub async fn set_voc_tuning_parameters(
                &mut self,
                parameter: VocTuning,
            ) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<20>(
                    Command::SetReadVocTuningParameters,
                    Some(&(<[u16; 6]>::from(parameter))),
                )
                .await
            }

            /// Read the [`VocAlgorithmState`](crate::data::VocAlgorithmState) parameters
            /// from the sensor.
            /// The VOC algorithm state is lost after a device reset or power cycle, this enables
            /// storing it persistently and using
            /// [`set_voc_algorithm_state`](Sen66::set_voc_algorithm_state) to restore it.
            /// Can be read every measurement.
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_voc_algorithm_state(
                &mut self,
            ) -> Result<VocAlgorithmState, Sen66Error<ERR>> {
                let received = self
                    .write_read::<2, 12>(Command::SetReadVocAlgorithmState, None)
                    .await?;
                Ok(VocAlgorithmState::try_from(&received[..])?)
            }

            /// Set the [`VocAlgorithmState`](crate::data::VocAlgorithmState) parameters
            /// for the sensor.
            /// Use [`get_voc_algorithm_state`](Sen66::get_voc_algorithm_state) to retrive it.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Measuring state.
            pub async fn set_voc_algorithm_state(
                &mut self,
                parameter: VocAlgorithmState,
            ) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<14>(
                    Command::SetReadVocAlgorithmState,
                    Some(&(<[u16; 4]>::from(parameter))),
                )
                .await
            }

            /// Read the [`NoxTuning`](crate::configuration::NoxTuning) parameters from the sensor.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_nox_tuning_parameters(
                &mut self,
            ) -> Result<NoxTuning, Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                let received = self
                    .write_read::<2, 18>(Command::SetReadNoxTuningParameters, None)
                    .await?;
                Ok(NoxTuning::try_from(&received[..])?)
            }

            /// Set the [`NoxTuning`](crate::configuration::NoxTuning) parameters for the sensor.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            pub async fn set_nox_tuning_parameters(
                &mut self,
                parameter: NoxTuning,
            ) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<20>(
                    Command::SetReadNoxTuningParameters,
                    Some(&(<[u16; 6]>::from(parameter))),
                )
                .await
            }

            /// Execute the forced recalibration (FRC) for the CO2 sensor.
            /// Wait at least 1000ms after power-on or 600ms after stopping the measurement before
            /// issuing this command.
            /// Execution Time: 500ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn perform_forced_co2_recalibration(
                &mut self,
                parameter: TargetCO2Concentration,
            ) -> Result<Co2Correction, Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                let received = self
                    .write_read::<5, 3>(
                        Command::ForcedRecalibration,
                        Some(&([u16::from(parameter)])),
                    )
                    .await?;
                let value = Co2Correction::try_from(&received[..])?;
                if !value.is_valid() {
                    Err(Sen66Error::FailedCo2Recalibration)
                } else {
                    Ok(value)
                }
            }

            /// Read out whether the automatic self calibration (ASC) for the CO2 sensor is
            /// enabled or disabled.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_co2_asc_state(&mut self) -> Result<AscState, Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                let received = self
                    .write_read::<2, 3>(Command::SetReadCo2AutomaticSelfCalibration, None)
                    .await?;
                Ok(AscState::try_from(&received[..])?)
            }

            /// Set whether the automatic self calibration (ASC) for the CO2 sensor is
            /// enabled or disabled.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            pub async fn set_co2_asc_state(
                &mut self,
                new_state: AscState,
            ) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<5>(
                    Command::SetReadCo2AutomaticSelfCalibration,
                    Some(&([u16::from(new_state)])),
                )
                .await
            }

            /// Read the configured ambient pressure for CO2 sensor compensation from the sensor.
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_ambient_pressure(
                &mut self,
            ) -> Result<AmbientPressure, Sen66Error<ERR>> {
                let received = self
                    .write_read::<2, 3>(Command::SetReadAmbientPreassure, None)
                    .await?;
                Ok(AmbientPressure::try_from(&received[..])?)
            }

            /// Configure the ambient pressure for CO2 sensor compensation for the sensor.
            /// Execution Time: 20ms
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            pub async fn set_ambient_pressure(
                &mut self,
                parameter: AmbientPressure,
            ) -> Result<(), Sen66Error<ERR>> {
                self.write::<5>(
                    Command::SetReadAmbientPreassure,
                    Some(&([u16::from(parameter)])),
                )
                .await
            }

            /// Read the configured sensor altitude for CO2 sensor compensation from the sensor.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            /// - [`DataError`](crate::error::Sen66Error::DataError): If the received data is
            /// corrupted or wrong.
            pub async fn get_sensor_altitude(&mut self) -> Result<SensorAltitude, Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                let received = self
                    .write_read::<2, 3>(Command::SetReadSensorAltitude, None)
                    .await?;
                Ok(SensorAltitude::try_from(&received[..])?)
            }

            /// Configure the sensor altitude for CO2 sensor compensation for the sensor.
            /// Execution Time: 20ms
            /// <div class="warning">Only available in idle state</div>
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            /// - [`WrongState`](crate::error::Sen66Error::WrongState): If the command is called in
            /// Idle state.
            pub async fn set_sensor_altitude(
                &mut self,
                parameter: SensorAltitude,
            ) -> Result<(), Sen66Error<ERR>> {
                if self.state != SensorState::Idle {
                    return Err(Sen66Error::WrongState("Measuring"));
                }
                self.write::<5>(
                    Command::SetReadSensorAltitude,
                    Some(&([u16::from(parameter)])),
                )
                .await
            }

            /// Closes the sensor interface, stops active measuring if active and returns the
            /// contained peripherals.
            ///
            /// # Errors
            ///
            /// - [`I2cError`](crate::error::Sen66Error::I2cError): If an error on the underlying
            /// I2C bus occurs.
            pub async fn shutdown(mut self) -> Result<(DELAY, I2C), Sen66Error<ERR>> {
                if self.state == SensorState::Measuring {
                    self.stop_measurement().await?;
                }
                Ok((self.delay, self.i2c))
            }

            /// Closes the sensor interface, does not change sensor state.
            pub async fn kill(self) -> (DELAY, I2C) {
                (self.delay, self.i2c)
            }

            /// Writes the command and optional data to the sensor, waits for the execution time of
            /// the command and reads the values returned.
            async fn write_read<const TX_SIZE: usize, const RX_SIZE: usize>(
                &mut self,
                command: Command,
                data: Option<&[u16]>,
            ) -> Result<[u8; RX_SIZE], Sen66Error<ERR>> {
                self.write::<TX_SIZE>(command, data).await?;
                Ok(self.read().await?)
            }

            /// Writes the command and optional data to the sensor and waits for the execution time
            /// of the command.
            async fn write<const TX_SIZE: usize>(
                &mut self,
                command: Command,
                data: Option<&[u16]>,
            ) -> Result<(), Sen66Error<ERR>> {
                let mut sent = [0; TX_SIZE];
                let command_data = command.to_be_bytes();
                sent[0] = command_data[0];
                sent[1] = command_data[1];

                let len = if let Some(data) = data {
                    for (i, datum) in data.iter().enumerate() {
                        let bytes = datum.to_be_bytes();
                        sent[2 + i * 3] = bytes[0];
                        sent[3 + i * 3] = bytes[1];
                        sent[4 + i * 3] = compute_crc8(&bytes);
                    }
                    2 + data.len() * 3
                } else {
                    2
                };
                self.i2c.write(ADDRESS | WRITE_FLAG, &sent[..len]).await?;
                self.delay.delay_ms(command.execution_time_ms()).await;
                Ok(())
            }

            /// Reads data from the I2C bus.
            async fn read<const RX_SIZE: usize>(
                &mut self,
            ) -> Result<[u8; RX_SIZE], Sen66Error<ERR>> {
                let mut received = [0; RX_SIZE];
                self.i2c.read(ADDRESS | READ_FLAG, &mut received).await?;
                Ok(received)
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use embedded_hal_mock::eh1::{
                delay::NoopDelay,
                i2c::{Mock as I2cMock, Transaction as I2cTransaction},
            };

            #[test_macro]
            async fn start_measurements_works() {
                let expected_transaction = [I2cTransaction::write(0x6B | 0x00, vec![0x00, 0x21])];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);

                sensor.start_measurement().await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn stop_measurement_in_idle_yields_error() {
                let expected_transaction = [];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);

                assert!(sensor.stop_measurement().await.is_err());
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn stop_measurement_works() {
                let expected_transaction = [I2cTransaction::write(0x6B | 0x00, vec![0x01, 0x04])];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                sensor.stop_measurement().await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn if_data_ready_is_data_ready_yields_ready() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x02, 0x02]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x00, 0x01, 0xB0]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert_eq!(sensor.is_data_ready().await.unwrap(), DataStatus::Ready);
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn if_data_not_ready_is_data_ready_yields_not_ready() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x02, 0x02]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x00, 0x00, 0x81]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert_eq!(sensor.is_data_ready().await.unwrap(), DataStatus::NotReady);
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn read_measured_values_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x03, 0x00]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A,
                            0x00, 0x64, 0xFE, 0x00, 0xC8, 0x7F, 0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A,
                            0x00, 0x01, 0xB0,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert_eq!(
                    sensor.read_measured_values().await.unwrap(),
                    Measurement {
                        pm1_0: 1.0,
                        pm2_5: 1.0,
                        pm4_0: 1.0,
                        pm10_0: 1.0,
                        relative_humidity: 1.0,
                        temperature: 1.0,
                        voc_index: 1.0,
                        nox_index: 1.0,
                        co2: 1,
                    }
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn read_measured_raw_values_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x04, 0x05]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            0x00, 0x64, 0xFe, 0x00, 0xC8, 0x7F, 0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A,
                            0x00, 0x01, 0xB0,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert_eq!(
                    sensor.read_measured_raw_values().await.unwrap(),
                    RawMeasurement {
                        relative_humidity: 1.0,
                        temperature: 1.0,
                        voc: 10,
                        nox: 10,
                        co2: 1,
                    }
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn read_number_concentrations_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x03, 0x16]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A, 0x00, 0x0A, 0x5A,
                            0x00, 0x0A, 0x5A,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert_eq!(
                    sensor.read_number_concentrations().await.unwrap(),
                    Concentrations {
                        pm0_5: 1.0,
                        pm1_0: 1.0,
                        pm2_5: 1.0,
                        pm4_0: 1.0,
                        pm10_0: 1.0,
                    },
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_temperature_offset_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![
                        0x60, 0xB2, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00,
                        0x00, 0x81,
                    ],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);

                let offset = TemperatureOffset::new(0, 0, 0, 0).unwrap();
                sensor.set_temperature_offset(offset).await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_temperature_acceleration_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![
                        0x61, 0x00, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00,
                        0x00, 0x81,
                    ],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);

                let acceleration = TemperatureAcceleration::new(0, 0, 0, 0).unwrap();
                sensor
                    .set_temperature_acceleration(acceleration)
                    .await
                    .unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_product_name_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0xD0, 0x14]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            'S' as u8, 'E' as u8, 0x83, 'N' as u8, '6' as u8, 0x06, '6' as u8,
                            '\0' as u8, 0x69, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                            0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                            0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                            0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert_eq!(
                    sensor.get_product_name().await.unwrap().get_name_buffer(),
                    [
                        'S' as u8, 'E' as u8, 'N' as u8, '6' as u8, '6' as u8, '\0' as u8
                    ]
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_serial_number_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0xD0, 0x33]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            'S' as u8, 'E' as u8, 0x83, 'N' as u8, '6' as u8, 0x06, '6' as u8,
                            '\0' as u8, 0x69, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                            0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                            0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                            0x00, 0x00, 0x81, 0x00, 0x00, 0x81,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert_eq!(
                    sensor
                        .get_serial_number()
                        .await
                        .unwrap()
                        .get_serial_buffer(),
                    [
                        'S' as u8, 'E' as u8, 'N' as u8, '6' as u8, '6' as u8, '\0' as u8
                    ]
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn read_device_status_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0xD2, 0x06]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x00, 0x00, 0x81, 0x00, 0x00, 0x81]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert!(
                    sensor
                        .read_device_status()
                        .await
                        .unwrap()
                        .has_error()
                        .is_ok()
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn read_and_clear_device_status_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0xD2, 0x10]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x00, 0x00, 0x81, 0x00, 0x00, 0x81]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Measuring;

                assert!(
                    sensor
                        .read_and_clear_device_status()
                        .await
                        .unwrap()
                        .has_error()
                        .is_ok()
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn reset_device_works() {
                let expected_transaction = [I2cTransaction::write(0x6B | 0x00, vec![0xD3, 0x04])];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);

                sensor.reset_device().await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn start_fan_cleaning_works() {
                let expected_transaction = [I2cTransaction::write(0x6B | 0x00, vec![0x56, 0x07])];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);

                sensor.start_fan_cleaning().await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn activate_sht_heater_works() {
                let expected_transaction = [I2cTransaction::write(0x6B | 0x00, vec![0x37, 0x30])];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);

                sensor.activate_sht_heater().await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_voc_tuning_parameters_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x60, 0xD0]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x00, 0x81,
                            0x00, 0x0A, 0x5A, 0x00, 0x01, 0xB0,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;

                assert_eq!(
                    sensor.get_voc_tuning_parameters().await.unwrap(),
                    VocTuning::new(1, 1, 1, 0, 10, 1).unwrap()
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_voc_tuning_parameters_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![
                        0x60, 0xD0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00,
                        0x00, 0x81, 0x00, 0x0A, 0x5A, 0x00, 0x01, 0xB0,
                    ],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;

                sensor
                    .set_voc_tuning_parameters(VocTuning::new(1, 1, 1, 0, 10, 1).unwrap())
                    .await
                    .unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_voc_algorithm_state_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x61, 0x81]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;

                assert_eq!(
                    <[u16; 4]>::from(sensor.get_voc_algorithm_state().await.unwrap()),
                    [0x0001, 0x0001, 0x0001, 0x0001]
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_voc_algorithm_state_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![
                        0x61, 0x81, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00,
                        0x01, 0xB0,
                    ],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;

                let state = VocAlgorithmState::try_from(
                    &(vec![
                        0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0,
                    ])[..],
                )
                .unwrap();
                sensor.set_voc_algorithm_state(state).await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_nox_tuning_parameters_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x60, 0xE1]),
                    I2cTransaction::read(
                        0x6B | 0x01,
                        vec![
                            0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x00, 0x81,
                            0x00, 0x32, 0x26, 0x00, 0x01, 0xB0,
                        ],
                    ),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;

                assert_eq!(
                    sensor.get_nox_tuning_parameters().await.unwrap(),
                    NoxTuning::new(1, 1, 1, 0, 1).unwrap()
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_nox_tuning_parameters_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![
                        0x60, 0xE1, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00, 0x01, 0xB0, 0x00,
                        0x00, 0x81, 0x00, 0x32, 0x26, 0x00, 0x01, 0xB0,
                    ],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;

                sensor
                    .set_nox_tuning_parameters(NoxTuning::new(1, 1, 1, 0, 1).unwrap())
                    .await
                    .unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn perform_forced_co2_recalibration_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x67, 0x07, 0x03, 0xE8, 0xD4]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x83, 0xE8, 0xF7]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                assert_eq!(
                    u16::from(
                        sensor
                            .perform_forced_co2_recalibration(TargetCO2Concentration::from(1000))
                            .await
                            .unwrap()
                    ),
                    1000
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_co2_asc_state_is_enabled_yields_enabled() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x67, 0x11]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x00, 0x01, 0xb0]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                assert_eq!(sensor.get_co2_asc_state().await.unwrap(), AscState::Enabled);
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_co2_asc_state_is_disabled_yields_disabled() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x67, 0x11]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x00, 0x00, 0x81]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                assert_eq!(
                    sensor.get_co2_asc_state().await.unwrap(),
                    AscState::Disabled
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_co2_asc_state_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![0x67, 0x11, 0x00, 0x01, 0xB0],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                sensor.set_co2_asc_state(AscState::Enabled).await.unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_ambient_pressure_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x67, 0x20]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x02, 0xBC, 0x9A]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                assert_eq!(
                    sensor.get_ambient_pressure().await.unwrap(),
                    AmbientPressure::try_from(700).unwrap()
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_ambient_pressure_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![0x67, 0x20, 0x02, 0xBC, 0x9A],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                sensor
                    .set_ambient_pressure(AmbientPressure::try_from(700).unwrap())
                    .await
                    .unwrap();
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn get_sensor_altitude_works() {
                let expected_transaction = [
                    I2cTransaction::write(0x6B | 0x00, vec![0x67, 0x36]),
                    I2cTransaction::read(0x6B | 0x01, vec![0x02, 0xBC, 0x9A]),
                ];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                assert_eq!(
                    sensor.get_sensor_altitude().await.unwrap(),
                    SensorAltitude::try_from(700).unwrap()
                );
                sensor.kill().await.1.done();
            }

            #[test_macro]
            async fn set_sensor_altitude_works() {
                let expected_transaction = [I2cTransaction::write(
                    0x6B | 0x00,
                    vec![0x67, 0x36, 0x02, 0xBC, 0x9A],
                )];
                let i2c = I2cMock::new(&expected_transaction);
                let delay = NoopDelay::new();
                let mut sensor = Sen66::new(delay, i2c);
                sensor.state = SensorState::Idle;
                sensor
                    .set_sensor_altitude(SensorAltitude::try_from(700).unwrap())
                    .await
                    .unwrap();
                sensor.kill().await.1.done();
            }
        }
    }

    #[cfg(feature=feature_)]
    pub use inner::*;
}
