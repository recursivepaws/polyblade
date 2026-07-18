use std::{
    cmp::{max, min},
    num::ParseIntError,
};

use iced::widget::shader::wgpu;
use ultraviolet::Vec4;

#[derive(Debug, Default)]
#[allow(clippy::upper_case_acronyms)]
pub struct HSL {
    /// Hue in 0-360 degree
    pub h: f32,
    /// Saturation in 0...1 (percent)
    pub s: f32,
    /// Luminosity in 0...1 (percent)
    pub l: f32,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::upper_case_acronyms)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl TryFrom<&str> for RGBA {
    type Error = ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let r = u8::from_str_radix(&value[1..3], 16)?;
        let g = u8::from_str_radix(&value[3..5], 16)?;
        let b = u8::from_str_radix(&value[5..7], 16)?;
        Ok(Self {
            r,
            g,
            b,
            a: u8::MAX,
        })
    }
}

impl HSL {
    #[allow(dead_code)]
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
    }
}

impl From<RGBA> for HSL {
    fn from(val: RGBA) -> Self {
        let mut h: f32;
        let (r, g, b) = (val.r, val.g, val.b);

        let max = max(max(r, g), b);
        let min = min(min(r, g), b);

        // Normalized RGB: Divide everything by 255 to get percentages of colors.
        let (r, g, b) = (r as f32 / 255_f32, g as f32 / 255_f32, b as f32 / 255_f32);
        let (min, max) = (min as f32 / 255_f32, max as f32 / 255_f32);

        // Luminosity is the average of the max and min rgb color intensities.
        let l: f32 = (max + min) / 2_f32;

        // Saturation
        let delta: f32 = max - min;
        if delta == 0_f32 {
            // it's gray
            return HSL {
                h: 0_f32,
                s: 0_f32,
                l,
            };
        }

        // it's not gray
        let s = if l < 0.5_f32 {
            delta / (max + min)
        } else {
            delta / (2_f32 - max - min)
        };

        // Hue
        let r2 = (((max - r) / 6_f32) + (delta / 2_f32)) / delta;
        let g2 = (((max - g) / 6_f32) + (delta / 2_f32)) / delta;
        let b2 = (((max - b) / 6_f32) + (delta / 2_f32)) / delta;

        h = match max {
            x if x == r => b2 - g2,
            x if x == g => (1_f32 / 3_f32) + r2 - b2,
            _ => (2_f32 / 3_f32) + g2 - r2,
        };

        // Fix wraparounds
        if h < 0 as f32 {
            h += 1_f32;
        } else if h > 1_f32 {
            h -= 1_f32;
        }

        // Hue is precise to milli-degrees, e.g. `74.52deg`.
        let h = (h * 360_f32 * 100_f32).round() / 100_f32;
        HSL { h, s, l }
    }
}

impl From<HSL> for RGBA {
    fn from(val: HSL) -> Self {
        if val.s == 0.0 {
            // Achromatic, i.e., grey.
            let l = percent_to_byte(val.l);
            return RGBA::new(l, l, l, u8::MAX);
        }

        let h = val.h / 360.0; // treat this as 0..1 instead of degrees
        let s = val.s;
        let l = val.l;

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - (l * s)
        };
        let p = 2.0 * l - q;

        RGBA::new(
            percent_to_byte(hue_to_rgb(p, q, h + 1.0 / 3.0)),
            percent_to_byte(hue_to_rgb(p, q, h)),
            percent_to_byte(hue_to_rgb(p, q, h - 1.0 / 3.0)),
            u8::MAX,
        )
    }
}

impl From<iced::Color> for RGBA {
    fn from(value: iced::Color) -> Self {
        Self {
            r: (value.r * 255.0) as u8,
            g: (value.g * 255.0) as u8,
            b: (value.b * 255.0) as u8,
            a: (value.a * 255.0) as u8,
        }
    }
}
impl From<RGBA> for iced::Color {
    fn from(value: RGBA) -> Self {
        Self {
            r: value.r as f32 / 255.0,
            g: value.g as f32 / 255.0,
            b: value.b as f32 / 255.0,
            a: value.a as f32 / 255.0,
        }
    }
}
impl From<RGBA> for iced::Background {
    fn from(value: RGBA) -> Self {
        iced::Color::from(value).into()
    }
}

fn srgb(x: f32) -> f32 {
    ((x / 255.0 + 0.055) / 1.055).powf(2.4)
}

impl From<RGBA> for Vec4 {
    fn from(value: RGBA) -> Self {
        Vec4::new(
            srgb(value.r as f32),
            srgb(value.g as f32),
            srgb(value.b as f32),
            srgb(value.a as f32),
        )
    }
}

impl From<RGBA> for wgpu::Color {
    fn from(val: RGBA) -> Self {
        wgpu::Color {
            r: val.r as f64 / 255.0,
            g: val.g as f64 / 255.0,
            b: val.b as f64 / 255.0,
            a: val.a as f64 / 255.0,
        }
    }
}
impl From<HSL> for wgpu::Color {
    fn from(val: HSL) -> Self {
        Into::<RGBA>::into(val).into()
    }
}

fn percent_to_byte(percent: f32) -> u8 {
    (percent * 255.0).round() as u8
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    // Normalize
    let t = if t < 0.0 {
        t + 1.0
    } else if t > 1.0 {
        t - 1.0
    } else {
        t
    };

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
