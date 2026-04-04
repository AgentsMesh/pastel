/// Source location for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

/// A token with its kind and source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // -- Literals --
    Integer(i64),
    Float(f64),
    String(String),
    Color(String), // e.g. "FF0066" or "FF006680" (without #)
    Bool(bool),

    // -- Identifiers & Keywords --
    Ident(String),
    Canvas,
    Asset,
    Let,
    Include,
    Frame,
    Text,
    Image,
    Shape,
    Component,
    Use,
    Page,

    // -- Punctuation --
    LBrace,    // {
    RBrace,    // }
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    Equals,    // =
    Comma,     // ,
    Dot,       // .

    // -- Special --
    Eof,
}

impl TokenKind {
    /// Map a keyword string to its token kind, or None if not a keyword.
    pub fn keyword(s: &str) -> Option<TokenKind> {
        match s {
            "canvas" => Some(TokenKind::Canvas),
            "asset" => Some(TokenKind::Asset),
            "let" => Some(TokenKind::Let),
            "include" => Some(TokenKind::Include),
            "frame" => Some(TokenKind::Frame),
            "text" => Some(TokenKind::Text),
            "image" => Some(TokenKind::Image),
            "shape" => Some(TokenKind::Shape),
            "component" => Some(TokenKind::Component),
            "use" => Some(TokenKind::Use),
            "page" => Some(TokenKind::Page),
            "true" => Some(TokenKind::Bool(true)),
            "false" => Some(TokenKind::Bool(false)),
            _ => None,
        }
    }
}
