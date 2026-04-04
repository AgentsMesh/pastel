use serde::Serialize;

// Re-export types from extra module for backward compatibility.
pub use super::extra::*;

/// Validated, normalized color representation.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(into = "String")]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_hex(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color { r, g, b, a: 255 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color { r, g, b, a })
            }
            _ => None,
        }
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn transparent() -> Color {
        Color { r: 0, g: 0, b: 0, a: 0 }
    }

    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

impl From<Color> for String {
    fn from(c: Color) -> String {
        c.to_hex()
    }
}

// -- Layout Enums --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    Horizontal,
    Vertical,
    Grid,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

// -- Dimension --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Dimension {
    #[serde(rename = "number")]
    Fixed(f64),
    #[serde(rename = "fill")]
    Fill,
    #[serde(rename = "hug")]
    Hug,
}

// -- Visual Styles --

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GradientStop {
    pub color: Color,
    pub position: f64, // 0.0 to 100.0
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Fill {
    #[serde(rename = "solid")]
    Solid { color: Color },
    #[serde(rename = "linear-gradient")]
    LinearGradient {
        angle: f64,
        stops: Vec<GradientStop>,
    },
    #[serde(rename = "radial-gradient")]
    RadialGradient {
        cx: f64,
        cy: f64,
        stops: Vec<GradientStop>,
    },
    #[serde(rename = "transparent")]
    Transparent,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stroke {
    pub width: f64,
    pub color: Color,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dash: Option<[f64; 2]>,
}

// -- Blend Mode --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shadow {
    pub x: f64,
    pub y: f64,
    pub blur: f64,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CornerRadius(pub [f64; 4]); // [tl, tr, br, bl]

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Padding(pub [f64; 4]); // [top, right, bottom, left]

// -- Layout --

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Layout {
    pub mode: LayoutMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<Justify>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<u32>,
}
