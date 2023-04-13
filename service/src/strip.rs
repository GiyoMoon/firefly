use crate::led::Led;
use palette::{Gradient, LinSrgb, Srgb};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct Strip {
    spi: Spidev,
    pub leds: Vec<Led>,
    on: bool,
}

impl Strip {
    pub fn new(device: &str, freq: u32, amount_of_leds: usize) -> io::Result<Strip> {
        let mut spi = Spidev::open(device)?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(freq)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;
        Ok(Self {
            spi,
            leds: vec![Led::new(); amount_of_leds],
            on: true,
        })
    }

    pub fn fill(&mut self, led: Led) -> io::Result<()> {
        self.leds.fill(led);
        self.update()
    }

    pub fn on(&mut self) -> io::Result<()> {
        if self.on {
            return Ok(());
        }
        self.on = true;
        self.update()
    }

    pub fn off(&mut self) -> io::Result<()> {
        if !self.on {
            return Ok(());
        }
        self.on = false;
        self.update()
    }

    pub fn set_gradient(&mut self, colors: Vec<(u8, u8, u8)>) -> io::Result<()> {
        let gradient = Gradient::new(colors.into_iter().map(|rgb| {
            LinSrgb::new(
                rgb.0 as f32 / 255.0,
                rgb.1 as f32 / 255.0,
                rgb.2 as f32 / 255.0,
            )
        }));

        gradient
            .take(self.leds.len())
            .zip(&mut self.leds)
            .for_each(|(color, led)| {
                *led = Srgb::from_linear(color).into();
            });
        self.update()
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    fn update(&mut self) -> io::Result<()> {
        let mut led_data: Vec<u8> = self.raw_led_data().collect();
        led_data.insert(0, 0);
        self.spi.write_all(&led_data)?;
        thread::sleep(Duration::from_micros(80));
        Ok(())
    }

    fn raw_led_data(&self) -> impl Iterator<Item = u8> + '_ {
        self.leds
            .iter()
            .flat_map(|led| led.to_raw_led_bytes(self.on))
    }
}
