# dht11-rs

An asynchronous, `embassy-rs` compatible driver for the DHT11 temperature and humidity sensor, demonstrated on an STM32 microcontroller.

This project provides a simple, `async`-native implementation for reading data from a DHT11 sensor. It is designed to work with the `embassy` async embedded framework and uses `defmt` for logging.

## Features

- **Asynchronous:** Built with `async/await` for use with `embassy` executors.
- **STM32 Focused:** Uses `embassy-stm32` for GPIO control.
- **Robust:** Includes checksum validation and error handling for timeouts.
- **Reliable Timing:** Uses a critical section (`cortex_m::interrupt::free`) to handle the timing-sensitive protocol, preventing interference from the async scheduler.
- **Simple API:** A straightforward `read()` method returns temperature and humidity.

## Hardware Requirements

- An STM32-based development board (e.g., a Nucleo). The example uses `PA0`, but any 5V-tolerant GPIO pin will work.
- A DHT11 sensor module.
- An ST-Link or similar debug probe for flashing and logging.

## Wiring

Connect the DHT11 sensor to your STM32 board as follows. A pull-up resistor (4.7kΩ to 10kΩ) is required on the data line if your sensor module does not include one.

- **DHT11 VCC** → **3.3V or 5V** on the board
- **DHT11 GND** → **GND** on the board
- **DHT11 DATA** → **PA0** on the board (or your chosen GPIO pin)

## How to Use

The driver is implemented in `src/dht11.rs` and a usage example is shown in `src/main.rs`.

### 1. Initialize the GPIO Pin

The DHT11 requires communication over a single data line. An open-drain GPIO configuration is used to allow the sensor to pull the line low.

```rust
use embassy_stm32::gpio::{Level, OutputOpenDrain, Speed};

// In your main async function:
let p = embassy_stm32::init(config);
let dht_pin = OutputOpenDrain::new(p.PA0, Level::High, Speed::VeryHigh);
```

### 2. Create a Driver Instance

Create a new `Dht11` instance, passing the configured pin.

```rust
use crate::dht11::Dht11;

let mut sensor = Dht11::new(dht_pin);
```

### 3. Read Sensor Data

Call the `read()` method in a loop to get sensor readings. The method is `async` and returns a `Result` containing a `DhtReading` struct on success.

```rust
use embassy_time::{Duration, Timer};

loop {
    match sensor.read().await {
        Ok(reading) => {
            defmt::info!(
                "Temperature: {} C, Humidity: {} %",
                reading.temperature, reading.humidity
            );
        }
        Err(e) => {
            defmt::error!("Error reading DHT11: {:?}", e);
        }
    }
    // A 2-second delay is recommended between readings for the DHT11
    Timer::after(Duration::from_millis(2000)).await;
}
```

## Building and Running

This project is a standard embedded Rust application.

1.  **Build the project:**
    ```sh
    cargo build --release
    ```

2.  **Flash and run:**
    Use your preferred tool, such as `probe-rs` or `cargo-embed`.
    ```sh
    # Using probe-rs
    probe-rs run --chip <YOUR_CHIP> target/thumbv8m.main-none-eabihf/release/dht11-rs

    # Using cargo-embed
    cargo embed --release
    ```

3.  **View Logs:**
    The output is logged using `defmt`. You can view it using `defmt-print` or directly within `cargo-embed`.
    ```sh
    # Example output
    INFO  Temperature: 24 C, Humidity: 45 %
    ```

## Driver Implementation Notes

The `dht11.rs` driver handles the DHT11's custom single-wire protocol.

- **Start Signal:** The MCU initiates communication by pulling the data line low for ~20ms.
- **Timing-Critical Section:** To reliably measure the short high/low pulses from the sensor that represent data bits, the reading logic is executed within a `cortex_m::interrupt::free` critical section. This ensures the async scheduler does not preempt the task during the sensitive timing operation.
- **Checksum:** The driver validates the checksum provided by the sensor to ensure data integrity. If it fails, a `DhtError::ChecksumMismatch` is returned.
