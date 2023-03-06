use bitvec::{order::Msb0, view::BitView};
use palette::rgb::Rgb;

/// High bit (logical 1) representation for SPI
const BIT_HIGH: u8 = 0b11110000;
/// Low bit (logical 0) representation for SPI
const BIT_LOW: u8 = 0b11000000;

#[derive(Debug, Clone, Default)]
pub struct Led {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub w: u8,
}

impl Led {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Led {
            r,
            g,
            b,
            ..Default::default()
        }
    }

    pub fn from_w(w: u8) -> Self {
        Led {
            w,
            ..Default::default()
        }
    }

    pub fn to_raw_led_bytes(&self, on: bool) -> Vec<u8> {
        let bytes = if on {
            // Somehow it's grbw even the specifications say rgbw?
            [self.g, self.r, self.b, self.w]
        } else {
            [0, 0, 0, 0]
        };

        bytes
            .view_bits::<Msb0>()
            .iter()
            .map(|bit| match *bit {
                true => BIT_HIGH,
                false => BIT_LOW,
            })
            .collect()
    }
}

impl From<[u8; 3]> for Led {
    fn from(colors: [u8; 3]) -> Self {
        Led {
            r: colors[0],
            g: colors[1],
            b: colors[2],
            ..Default::default()
        }
    }
}

impl From<Rgb> for Led {
    fn from(color: Rgb) -> Self {
        [
            (color.red * (u8::MAX as f32)) as u8,
            (color.green * (u8::MAX as f32)) as u8,
            (color.blue * (u8::MAX as f32)) as u8,
        ]
        .into()
    }
}
