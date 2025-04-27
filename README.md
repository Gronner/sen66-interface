# SEN66 Driver

[![Crates.io Version](https://img.shields.io/crates/v/sen66-interface?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fsen66-interface)](https://crates.io/crates/sen66-interface)
[![docs.rs](https://img.shields.io/docsrs/sen66-interface?logo=https%3A%2F%2Fdocs.rs%2Fsen66-interface%2F1.0.0%2Fscd30_interface%2F)](https://docs.rs/sen66-interface/1.0.0/scd30_interface/)
[![Integration Pipeline](https://github.com/Gronner/sen66-interface/actions/workflows/integration.yaml/badge.svg)](https://github.com/Gronner/sen66-interface/actions/workflows/integration.yaml)
[![codecov](https://codecov.io/gh/Gronner/sen66-interface/graph/badge.svg?token=NH6UCHBL19)](https://codecov.io/gh/Gronner/sen66-interface)
![Crates.io MSRV](https://img.shields.io/crates/msrv/sen66-interface)

A driver for interacting with Sensirion's [SEN66](https://sensirion.com/products/catalog/SEN66)
environment sensing platform via I2C. The driver is based on the
[embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/) traits and offers a
synchronous and asynchronous interface.

Provides a full implementation of the SEN66 features:

* Measure environment parameters:
    * Mass concentrations PM1.0, PM2.5, PM4.0, PM10.0 in ug/m³
    * Number concentrations PM0.5, PM1.0, PM2.5, PM4.0, PM10.0 in p/cm³
    * Ambient relative humidity in %
    * Ambient temperature in °C
    * [VOC Index](https://sensirion.com/media/documents/02232963/6294E043/Info_Note_VOC_Index.pdf) around 100
    * [NOx Index](https://sensirion.com/media/documents/9F289B95/6294DFFC/Info_Note_NOx_Index.pdf) around 1
    * Raw VOC ticks
    * Raw NOx ticks
    * CO2 in ppm
* Configure VOC, NOx, CO2 and Temperature determination
* Perform forced CO2 recalibration, SHT heating and fan cleaning
* Read out device information
    * Serial Number
    * Product Name
    * Device Status

## Example

This example showcases how to use the SEN66 with a ESP32-C6-DevKitM-1 using
[embassy](https://github.com/embassy-rs/embassy).

```rust, ignore
use sen66_interface::asynch::Sen66;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let i2c = esp_hal::i2c::master::I2c::new(
        peripherals.I2C0,
        esp_hal::i2c::master::Config::default()
        ).unwrap()
        .into_async()
        .with_sda(peripherals.GPIO22)
        .with_scl(peripherals.GPIO23);

    let mut sensor = Sen66::new(embassy_time::Delay, i2c);

    // Provide enough time to start up after power-on
    embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;

    sensor.get_product_name().await.unwrap()
}
```

## Feature Flags

* `async`: Provides an async interface, enabled by default.
* `blocking`: Provides a blocking interface.
* `defmt`: Provides support for defmt.


## Contributing

If you want to contribute, open a Pull Request with your suggested changes and ensure that the
integration pipeline runs.

* Commits should adhere to the [Conventional Commits specification](https://www.conventionalcommits.org/en/v1.0.0/#specification)
* The integration pipeline must pass.
* Test coverage should not degrade.
* At least one review is required

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT licenses ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

`SPDX-License-Identifier: Apache-2.0 OR MIT`
