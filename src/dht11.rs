use embassy_stm32::gpio::{OutputOpenDrain, Level};
use embassy_time::{Duration, Timer, Instant};
use cortex_m::interrupt;

#[derive(Debug, defmt::Format)]
pub enum DhtError {
    Timeout,
    ChecksumMismatch,
}

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct DhtReading {
    pub temperature: u8,
    pub humidity: u8,
}

pub struct Dht11<'d> {
    pin: OutputOpenDrain<'d>,
}

impl<'d> Dht11<'d> {
    pub fn new(pin: OutputOpenDrain<'d>) -> Self {
        Self { pin }
    }

    pub async fn read(&mut self) -> Result<DhtReading, DhtError> {
        let mut buffer = [0u8; 5];

        // 1. Start Signal (Async)
        self.pin.set_low();
        Timer::after(Duration::from_millis(20)).await;
        self.pin.set_high();

        // 2. Critical Timing Section
        let result = interrupt::free(|_cs| {
            let mut byte_idx = 0;
            let mut bit_mask = 0x80;

            // Simple timeout helper
            let wait_level = |target: Level, limit_us: u64| -> Result<(), DhtError> {
                let start = Instant::now();
                while self.pin.get_level() != target {
                    if start.elapsed().as_micros() > limit_us {
                        return Err(DhtError::Timeout);
                    }
                }
                Ok(())
            };

            // Handshake: Wait for Low (response start) -> High -> Low (start of data)
            wait_level(Level::Low, 60)?;
            wait_level(Level::High, 100)?;
            wait_level(Level::Low, 100)?;

            // Read 40 Bits
            for _ in 0..40 {
                // Wait for the 50us Low sync pulse to end
                wait_level(Level::High, 80)?;

                // Measure the High data pulse
                let start = Instant::now();
                while self.pin.get_level() == Level::High {
                    if start.elapsed().as_micros() > 100 {
                        return Err(DhtError::Timeout);
                    }
                }
                let duration = start.elapsed().as_micros();

                // Bit Logic: 26-28us is '0', 70us is '1'
                // We use 48us as a safe threshold
                if duration > 48 {
                    buffer[byte_idx] |= bit_mask;
                }

                bit_mask >>= 1;
                if bit_mask == 0 {
                    bit_mask = 0x80;
                    byte_idx += 1;
                }
            }
            Ok(())
        });

        result?;

        // 3. Checksum
        let sum: u16 = buffer[0] as u16 + buffer[1] as u16 + buffer[2] as u16 + buffer[3] as u16;
        if (sum & 0xFF) as u8 != buffer[4] {
            defmt::error!("Checksum Fail: Read {} | Calc {} | Raw: {:?}", buffer[4], sum & 0xFF, buffer);
            return Err(DhtError::ChecksumMismatch);
        }

        Ok(DhtReading {
            humidity: buffer[0],
            temperature: buffer[2],
        })
    }
}
