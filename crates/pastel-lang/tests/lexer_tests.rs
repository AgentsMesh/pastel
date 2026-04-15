use pastel_lang::error::ErrorKind;
use pastel_lang::lexer::Lexer;
use pastel_lang::token::TokenKind;

// ── Helper ──────────────────────────────────────────────────────────────

fn tokenize(src: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(src);
    lexer
        .tokenize()
        .expect("tokenize should succeed")
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

fn tokenize_err(src: &str) -> ErrorKind {
    let mut lexer = Lexer::new(src);
    lexer.tokenize().unwrap_err().kind
}

// ── Color literals ──────────────────────────────────────────────────────

#[test]
fn color_6_digit() {
    let kinds = tokenize("#FF0066");
    assert_eq!(
        kinds,
        vec![TokenKind::Color("FF0066".into()), TokenKind::Eof]
    );
}

#[test]
fn color_8_digit_with_alpha() {
    let kinds = tokenize("#FF006680");
    assert_eq!(
        kinds,
        vec![TokenKind::Color("FF006680".into()), TokenKind::Eof]
    );
}

#[test]
fn color_lowercase() {
    let kinds = tokenize("#aabbcc");
    assert_eq!(
        kinds,
        vec![TokenKind::Color("aabbcc".into()), TokenKind::Eof]
    );
}

#[test]
fn color_invalid_3_digit() {
    let err = tokenize_err("#FFF");
    assert_eq!(err, ErrorKind::InvalidColor);
}

#[test]
fn color_invalid_non_hex() {
    let err = tokenize_err("#GGGGGG");
    assert_eq!(err, ErrorKind::InvalidColor);
}

#[test]
fn color_invalid_5_digit() {
    let err = tokenize_err("#12345");
    assert_eq!(err, ErrorKind::InvalidColor);
}

// ── Number literals ─────────────────────────────────────────────────────

#[test]
fn integer_positive() {
    let kinds = tokenize("42");
    assert_eq!(kinds, vec![TokenKind::Integer(42), TokenKind::Eof]);
}

#[test]
fn integer_zero() {
    let kinds = tokenize("0");
    assert_eq!(kinds, vec![TokenKind::Integer(0), TokenKind::Eof]);
}

#[test]
fn integer_negative() {
    let kinds = tokenize("-10");
    assert_eq!(kinds, vec![TokenKind::Integer(-10), TokenKind::Eof]);
}

#[test]
fn float_positive() {
    let kinds = tokenize("3.5");
    assert_eq!(kinds, vec![TokenKind::Float(3.5), TokenKind::Eof]);
}

#[test]
fn float_negative() {
    let kinds = tokenize("-0.5");
    assert_eq!(kinds, vec![TokenKind::Float(-0.5), TokenKind::Eof]);
}

#[test]
fn float_leading_zero() {
    let kinds = tokenize("0.75");
    assert_eq!(kinds, vec![TokenKind::Float(0.75), TokenKind::Eof]);
}

// ── String literals ─────────────────────────────────────────────────────

#[test]
fn string_basic() {
    let kinds = tokenize(r#""hello""#);
    assert_eq!(
        kinds,
        vec![TokenKind::String("hello".into()), TokenKind::Eof]
    );
}

#[test]
fn string_empty() {
    let kinds = tokenize(r#""""#);
    assert_eq!(kinds, vec![TokenKind::String("".into()), TokenKind::Eof]);
}

#[test]
fn string_escape_newline() {
    let kinds = tokenize(r#""line\nbreak""#);
    assert_eq!(
        kinds,
        vec![TokenKind::String("line\nbreak".into()), TokenKind::Eof]
    );
}

#[test]
fn string_escape_tab() {
    let kinds = tokenize(r#""col\there""#);
    assert_eq!(
        kinds,
        vec![TokenKind::String("col\there".into()), TokenKind::Eof]
    );
}

#[test]
fn string_escape_backslash() {
    let kinds = tokenize(r#""back\\slash""#);
    assert_eq!(
        kinds,
        vec![TokenKind::String("back\\slash".into()), TokenKind::Eof]
    );
}

#[test]
fn string_escape_quote() {
    let kinds = tokenize(r#""say \"hi\"""#);
    assert_eq!(
        kinds,
        vec![TokenKind::String("say \"hi\"".into()), TokenKind::Eof]
    );
}

#[test]
fn string_unterminated() {
    let err = tokenize_err(r#""no closing"#);
    assert_eq!(err, ErrorKind::UnterminatedString);
}

// ── Identifiers ─────────────────────────────────────────────────────────

#[test]
fn ident_simple() {
    let kinds = tokenize("foo");
    assert_eq!(kinds, vec![TokenKind::Ident("foo".into()), TokenKind::Eof]);
}

#[test]
fn ident_with_hyphens() {
    let kinds = tokenize("nav-links");
    assert_eq!(
        kinds,
        vec![TokenKind::Ident("nav-links".into()), TokenKind::Eof]
    );
}

#[test]
fn ident_with_underscores() {
    let kinds = tokenize("_private");
    assert_eq!(
        kinds,
        vec![TokenKind::Ident("_private".into()), TokenKind::Eof]
    );
}

#[test]
fn ident_alphanumeric() {
    let kinds = tokenize("btn2");
    assert_eq!(kinds, vec![TokenKind::Ident("btn2".into()), TokenKind::Eof]);
}

// ── Keywords ────────────────────────────────────────────────────────────

#[test]
fn keyword_canvas() {
    let kinds = tokenize("canvas");
    assert_eq!(kinds, vec![TokenKind::Canvas, TokenKind::Eof]);
}

#[test]
fn keyword_asset() {
    let kinds = tokenize("asset");
    assert_eq!(kinds, vec![TokenKind::Asset, TokenKind::Eof]);
}

#[test]
fn keyword_let() {
    let kinds = tokenize("let");
    assert_eq!(kinds, vec![TokenKind::Let, TokenKind::Eof]);
}

#[test]
fn keyword_include() {
    let kinds = tokenize("include");
    assert_eq!(kinds, vec![TokenKind::Include, TokenKind::Eof]);
}

#[test]
fn keyword_frame() {
    let kinds = tokenize("frame");
    assert_eq!(kinds, vec![TokenKind::Frame, TokenKind::Eof]);
}

#[test]
fn keyword_text() {
    let kinds = tokenize("text");
    assert_eq!(kinds, vec![TokenKind::Text, TokenKind::Eof]);
}

#[test]
fn keyword_image() {
    let kinds = tokenize("image");
    assert_eq!(kinds, vec![TokenKind::Image, TokenKind::Eof]);
}

#[test]
fn keyword_shape() {
    let kinds = tokenize("shape");
    assert_eq!(kinds, vec![TokenKind::Shape, TokenKind::Eof]);
}

#[test]
fn keyword_component() {
    let kinds = tokenize("component");
    assert_eq!(kinds, vec![TokenKind::Component, TokenKind::Eof]);
}

#[test]
fn keyword_use() {
    let kinds = tokenize("use");
    assert_eq!(kinds, vec![TokenKind::Use, TokenKind::Eof]);
}

#[test]
fn keyword_true() {
    let kinds = tokenize("true");
    assert_eq!(kinds, vec![TokenKind::Bool(true), TokenKind::Eof]);
}

#[test]
fn keyword_false() {
    let kinds = tokenize("false");
    assert_eq!(kinds, vec![TokenKind::Bool(false), TokenKind::Eof]);
}

// ── Punctuation ─────────────────────────────────────────────────────────

#[test]
fn punctuation_braces() {
    let kinds = tokenize("{ }");
    assert_eq!(
        kinds,
        vec![TokenKind::LBrace, TokenKind::RBrace, TokenKind::Eof]
    );
}

#[test]
fn punctuation_parens() {
    let kinds = tokenize("( )");
    assert_eq!(
        kinds,
        vec![TokenKind::LParen, TokenKind::RParen, TokenKind::Eof]
    );
}

#[test]
fn punctuation_brackets() {
    let kinds = tokenize("[ ]");
    assert_eq!(
        kinds,
        vec![TokenKind::LBracket, TokenKind::RBracket, TokenKind::Eof]
    );
}

#[test]
fn punctuation_equals_comma_dot() {
    let kinds = tokenize("= , .");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Equals,
            TokenKind::Comma,
            TokenKind::Dot,
            TokenKind::Eof,
        ]
    );
}

// ── Comments ────────────────────────────────────────────────────────────

#[test]
fn line_comment_skipped() {
    let kinds = tokenize("// this is a comment\n42");
    assert_eq!(kinds, vec![TokenKind::Integer(42), TokenKind::Eof]);
}

#[test]
fn comment_at_end_of_line() {
    let kinds = tokenize("42 // trailing");
    assert_eq!(kinds, vec![TokenKind::Integer(42), TokenKind::Eof]);
}

#[test]
fn only_comment() {
    let kinds = tokenize("// just a comment");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

// ── Whitespace ──────────────────────────────────────────────────────────

#[test]
fn tabs_and_spaces() {
    let kinds = tokenize("  \t  42  \t  ");
    assert_eq!(kinds, vec![TokenKind::Integer(42), TokenKind::Eof]);
}

#[test]
fn newlines_between_tokens() {
    let kinds = tokenize("42\n\n100");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer(42),
            TokenKind::Integer(100),
            TokenKind::Eof
        ]
    );
}

// ── Edge cases ──────────────────────────────────────────────────────────

#[test]
fn empty_input() {
    let kinds = tokenize("");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn only_whitespace() {
    let kinds = tokenize("   \n\t  \n  ");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn only_comments_and_whitespace() {
    let kinds = tokenize("// comment 1\n  // comment 2\n");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn unexpected_character() {
    let err = tokenize_err("@");
    assert_eq!(err, ErrorKind::UnexpectedChar);
}

// ── Multi-token sequences ───────────────────────────────────────────────

#[test]
fn attribute_assignment() {
    let kinds = tokenize("width = 100");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("width".into()),
            TokenKind::Equals,
            TokenKind::Integer(100),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn color_attribute() {
    let kinds = tokenize("fill = #FF0066");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("fill".into()),
            TokenKind::Equals,
            TokenKind::Color("FF0066".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn array_literal() {
    let kinds = tokenize("[10, 20, 30]");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LBracket,
            TokenKind::Integer(10),
            TokenKind::Comma,
            TokenKind::Integer(20),
            TokenKind::Comma,
            TokenKind::Integer(30),
            TokenKind::RBracket,
            TokenKind::Eof,
        ]
    );
}

// ── Span tracking ───────────────────────────────────────────────────────

#[test]
fn span_line_tracking() {
    let mut lexer = Lexer::new("foo\nbar");
    let tokens = lexer.tokenize().unwrap();
    // "foo" on line 1
    assert_eq!(tokens[0].span.line, 1);
    // "bar" on line 2
    assert_eq!(tokens[1].span.line, 2);
}
