#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        }
    }
}

impl Color {
    pub fn parse(s: &str) -> Color {
        if s.len() != 7 || (s.len() == 7 && !s.starts_with('#')) {
            return Self::default();
        }
        if let Ok(s) = u32::from_str_radix(&s[1..], 16) {
            Color {
                r: ((s >> 16) & 0xFF) as u8,
                g: ((s >> 8) & 0xFF) as u8,
                b: (s & 0xFF) as u8,
            }
        } else {
            Self::default()
        }
    }
}
