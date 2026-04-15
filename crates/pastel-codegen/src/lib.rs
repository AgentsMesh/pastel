pub mod html;
pub mod react;
pub mod tokens;

/// Supported output formats for code generation.
pub enum Format {
    Tokens,
    Html,
    React,
}

impl Format {
    pub fn parse_str(s: &str) -> Option<Format> {
        match s.to_lowercase().as_str() {
            "tokens" => Some(Format::Tokens),
            "html" => Some(Format::Html),
            "react" => Some(Format::React),
            _ => None,
        }
    }
}
