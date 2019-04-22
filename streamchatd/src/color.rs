use twitchchat::RGB;

pub trait RelativeColor {
    fn is_dark(self) -> bool;
    fn is_light(self) -> bool;
}

impl RelativeColor for RGB {
    fn is_dark(self) -> bool {
        let HSL(.., l) = self.into();
        l < 30.0 // random number
    }

    fn is_light(self) -> bool {
        let HSL(.., l) = self.into();
        l < 80.0 // random number
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct HSL(pub f64, pub f64, pub f64); // H S L

impl std::fmt::Display for HSL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let HSL(h, s, l) = self;
        write!(f, "{:.2}%, {:.2}%, {:.2}%", h, s, l)
    }
}

impl From<RGB> for HSL {
    fn from(RGB(r, g, b): RGB) -> Self {
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
