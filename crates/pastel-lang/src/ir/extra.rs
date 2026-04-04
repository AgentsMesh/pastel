use serde::Serialize;

// -- Font Weight --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FontWeight {
    Thin,       // 100
    Light,      // 300
    Normal,     // 400
    Medium,     // 500
    Semibold,   // 600
    Bold,       // 700
    Extrabold,  // 800
    Black,      // 900
}

impl FontWeight {
    pub fn from_str(s: &str) -> Option<FontWeight> {
        match s.to_lowercase().as_str() {
            "thin" => Some(FontWeight::Thin),
            "light" => Some(FontWeight::Light),
            "normal" | "regular" => Some(FontWeight::Normal),
            "medium" => Some(FontWeight::Medium),
            "semibold" | "semi-bold" => Some(FontWeight::Semibold),
            "bold" => Some(FontWeight::Bold),
            "extrabold" | "extra-bold" => Some(FontWeight::Extrabold),
            "black" => Some(FontWeight::Black),
            _ => None,
        }
    }

    pub fn to_css_value(&self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::Semibold => 600,
            FontWeight::Bold => 700,
            FontWeight::Extrabold => 800,
            FontWeight::Black => 900,
        }
    }
}

// -- Text Align --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

// -- Text Decoration --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextDecoration {
    None,
    Underline,
    Strikethrough,
}

// -- Text Transform --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
}

// -- Image Fit --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFit {
    Cover,
    Contain,
    Fill,
    None,
}

// -- Position --

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionMode {
    Relative,
    Absolute,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position {
    pub mode: PositionMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<f64>,
}
