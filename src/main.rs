#![no_std]
#![no_main]

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::{Level, OutputOpenDrain, Speed};
use embassy_time::{Duration, Timer};

// Import the module
mod dht11;
use dht11::Dht11;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Use default config for U5 (Runs on MSI clock ~4MHz or 16MHz depending on chip defaults)
    // This avoids the complex PLL errors.
    let config = Config::default();

    let p = embassy_stm32::init(config);

    info!("Initializing DHT11...");

    // OutputOpenDrain::new(pin, initial_level, speed)
    let dht_pin = OutputOpenDrain::new(p.PA0, Level::High, Speed::VeryHigh);

    // Pass the pin directly
    let mut sensor = Dht11::new(dht_pin);

    loop {
        match sensor.read().await {
            Ok(reading) => {
                info!(
                    "Temperature: {} C, Humidity: {} %",
                    reading.temperature, reading.humidity
                );
            }
            Err(e) => {
                error!("Error reading DHT11: {:?}", e);
            }
        }

        Timer::after(Duration::from_millis(2000)).await;
    }
}
