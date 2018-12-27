use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color(pub u8, pub u8, pub u8);

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Color(r, g, b) = self;
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
    }
}

impl Color {
    pub fn is_dark(self) -> bool {
        let HSL(.., l) = self.into();
        l < 30.0
    }
}

impl Default for Color {
    fn default() -> Self {
        Color(0xFF, 0xFF, 0xFF)
    }
}

impl<'a> From<&'a str> for Color {
    fn from(s: &'a str) -> Self {
        let s = match (s.chars().next(), s.len()) {
            (Some('#'), 7) => &s[1..],
            (.., 6) => s,
            _ => {
                debug!("invalid color '{}' defaulting", s);
                return Self::default();
            }
        };

        u32::from_str_radix(&s, 16)
            .and_then(|s| {
                Ok(Color {
                    0: ((s >> 16) & 0xFF) as u8,
                    1: ((s >> 8) & 0xFF) as u8,
                    2: (s & 0xFF) as u8,
                })
            })
            .unwrap_or_default()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TwitchColor {
    Blue,
    BlueViolet,
    CadetBlue,
    Chocolate,
    Coral,
    DodgerBlue,
    Firebrick,
    GoldenRod,
    Green,
    HotPink,
    OrangeRed,
    Red,
    SeaGreen,
    SpringGreen,
    YellowGreen,
    Turbo(Color),
}

impl fmt::Display for TwitchColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TwitchColor::*;

        match self {
            Blue => write!(f, "Blue"),
            BlueViolet => write!(f, "Blue Violet"),
            CadetBlue => write!(f, "Cadet Blue"),
            Chocolate => write!(f, "Chocolate"),
            Coral => write!(f, "Coral"),
            DodgerBlue => write!(f, "Dodger Blue"),
            Firebrick => write!(f, "Firebrick"),
            GoldenRod => write!(f, "Golden Rod"),
            Green => write!(f, "Green"),
            HotPink => write!(f, "Hot Pink"),
            OrangeRed => write!(f, "Orange Red"),
            Red => write!(f, "Red"),
            SeaGreen => write!(f, "Sea Green"),
            SpringGreen => write!(f, "Spring Green"),
            YellowGreen => write!(f, "Yellow Green"),
            Turbo(color) => write!(f, "{}", color),
        }
    }
}

impl From<TwitchColor> for Color {
    fn from(c: TwitchColor) -> Self {
        if let TwitchColor::Turbo(rgb) = c {
            return rgb;
        }
        colors()
            .iter()
            .find(|(color, _)| *color == c)
            .map(|&(_, r)| r)
            .unwrap_or_default()
    }
}

impl From<Color> for TwitchColor {
    fn from(Color(r, g, b): Color) -> Self {
        colors()
            .iter()
            .find(|(_, rgb)| *rgb == Color(r, g, b))
            .map(|&(c, _)| c)
            .unwrap_or_else(|| TwitchColor::Turbo(Color(r, g, b)))
    }
}

impl<'a> From<&'a str> for TwitchColor {
    fn from(s: &'a str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "blue" => TwitchColor::Blue,
            "blue violet" | "blue_violet" | "blueviolet" => TwitchColor::BlueViolet,
            "cadet blue" | "cadet_blue" | "cadetblue" => TwitchColor::CadetBlue,
            "chocolate" => TwitchColor::Chocolate,
            "coral" => TwitchColor::Coral,
            "dodger blue" | "dodger_blue" | "dodgerblue" => TwitchColor::DodgerBlue,
            "firebrick" => TwitchColor::Firebrick,
            "golden rod" | "golden_rod" | "goldenrod" => TwitchColor::GoldenRod,
            "green" => TwitchColor::Green,
            "hot pink" | "hot_pink" | "hotpink" => TwitchColor::HotPink,
            "orange red" | "orange_red" | "orangered" => TwitchColor::OrangeRed,
            "red" => TwitchColor::Red,
            "sea green" | "sea_green" | "seagreen" => TwitchColor::SeaGreen,
            "spring green" | "spring_green" | "springgreen" => TwitchColor::SpringGreen,
            "yellow green" | "yellow_green" | "yellowgreen" => TwitchColor::YellowGreen,
            _ => TwitchColor::Turbo(Color::from(s)),
        }
    }
}

const fn colors() -> [(TwitchColor, Color); 15] {
    use self::TwitchColor::*;
    [
        (Blue, Color(0x00, 0x00, 0xFF)),
        (BlueViolet, Color(0xFF, 0x7F, 0x50)),
        (CadetBlue, Color(0x1E, 0x90, 0xFF)),
        (Chocolate, Color(0x00, 0xFF, 0x7F)),
        (Coral, Color(0x9A, 0xCD, 0x32)),
        (DodgerBlue, Color(0x00, 0x80, 0x00)),
        (Firebrick, Color(0xFF, 0x45, 0x00)),
        (GoldenRod, Color(0xFF, 0x00, 0x00)),
        (Green, Color(0xDA, 0xA5, 0x20)),
        (HotPink, Color(0xFF, 0x69, 0xB4)),
        (OrangeRed, Color(0x5F, 0x9E, 0xA0)),
        (Red, Color(0x2E, 0x8B, 0x57)),
        (SeaGreen, Color(0xD2, 0x69, 0x1E)),
        (SpringGreen, Color(0x8A, 0x2B, 0xE2)),
        (YellowGreen, Color(0xB2, 0x22, 0x22)),
    ]
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct HSL(f64, f64, f64); // H S L

impl From<Color> for HSL {
    fn from(Color(r, g, b): Color) -> Self {
        #![allow(clippy::unknown_clippy_lints, clippy::many_single_char_names)]
        use std::cmp::{max, min};

        let max = max(max(r, g), b);
        let min = min(min(r, g), b);
        let (r, g, b) = (
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        );

        let (min, max) = (f64::from(min) / 255.0, f64::from(max) / 255.0);
        let l = (max + min) / 2.0;
        let delta: f64 = max - min;
        // this checks for grey
        if delta == 0.0 {
            return HSL(0.0, 0.0, ((l * 100.0).round() / 100.0) * 100.0);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let r2 = (((max - r) / 6.0) + (delta / 2.0)) / delta;
        let g2 = (((max - g) / 6.0) + (delta / 2.0)) / delta;
        let b2 = (((max - b) / 6.0) + (delta / 2.0)) / delta;

        let h = match match max {
            x if (x - r).abs() < 0.001 => b2 - g2,
            x if (x - g).abs() < 0.001 => (1.0 / 3.0) + r2 - b2,
            _ => (2.0 / 3.0) + g2 - r2,
        } {
            h if h < 0.0 => h + 1.0,
            h if h > 1.0 => h - 1.0,
            h => h,
        };

        let h = (h * 360.0 * 100.0).round() / 100.0;
        let s = ((s * 100.0).round() / 100.0) * 100.0;
        let l = ((l * 100.0).round() / 100.0) * 100.0;

        HSL(h, s, l)
    }
}
