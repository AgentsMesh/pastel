use crate::error::{ErrorKind, PastelError};
use crate::token::{Token, TokenKind};

use super::Lexer;

impl Lexer {
    pub(super) fn scan_token(&mut self) -> Result<Token, PastelError> {
        let ch = self.peek();
        match ch {
            // Punctuation
            '{' => self.single_char_token(TokenKind::LBrace),
            '}' => self.single_char_token(TokenKind::RBrace),
            '(' => self.single_char_token(TokenKind::LParen),
            ')' => self.single_char_token(TokenKind::RParen),
            '[' => self.single_char_token(TokenKind::LBracket),
            ']' => self.single_char_token(TokenKind::RBracket),
            '=' => self.single_char_token(TokenKind::Equals),
            ',' => self.single_char_token(TokenKind::Comma),
            '.' => self.single_char_token(TokenKind::Dot),

            // Color literal: #RRGGBB or #RRGGBBAA
            '#' => self.scan_color(),

            // String literal
            '"' => self.scan_string(),

            // Number literal (or negative number)
            c if c.is_ascii_digit() => self.scan_number(),
            '-' if self.peek_next().is_some_and(|c| c.is_ascii_digit()) => self.scan_number(),

            // Identifier or keyword
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier(),

            _ => {
                let span = self.current_span(0);
                self.advance();
                Err(PastelError::new(
                    ErrorKind::UnexpectedChar,
                    format!("unexpected character '{}'", ch),
                )
                .with_span(span))
            }
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Result<Token, PastelError> {
        self.advance();
        Ok(Token {
            kind,
            span: self.current_span(1),
        })
    }

    fn scan_color(&mut self) -> Result<Token, PastelError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        self.advance(); // skip '#'

        let mut hex = String::new();
        while !self.is_at_end() && self.peek().is_ascii_hexdigit() {
            hex.push(self.advance());
        }

        if hex.len() != 6 && hex.len() != 8 {
            return Err(PastelError::new(
                ErrorKind::InvalidColor,
                format!(
                    "color literal must be 6 or 8 hex digits, got {} digits",
                    hex.len()
                ),
            )
            .with_span(crate::token::Span {
                start: start_pos,
                end: self.pos,
                line: start_line,
                col: start_col,
            })
            .with_hint("use #RRGGBB or #RRGGBBAA format"));
        }

        let len = hex.len() + 1; // +1 for '#'
        Ok(Token {
            kind: TokenKind::Color(hex),
            span: self.current_span(len),
        })
    }

    fn scan_string(&mut self) -> Result<Token, PastelError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        self.advance(); // skip opening '"'

        let mut value = String::new();
        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            }
            if ch == '\\' && !self.is_at_end() {
                let escaped = self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    _ => {
                        value.push('\\');
                        value.push(escaped);
                    }
                }
            } else {
                value.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(PastelError::new(
                ErrorKind::UnterminatedString,
                "unterminated string literal",
            )
            .with_span(crate::token::Span {
                start: start_pos,
                end: self.pos,
                line: start_line,
                col: start_col,
            })
            .with_hint("add a closing '\"'"));
        }

        self.advance(); // skip closing '"'
        let len = self.pos - start_pos;
        Ok(Token {
            kind: TokenKind::String(value),
            span: self.current_span(len),
        })
    }

    fn scan_number(&mut self) -> Result<Token, PastelError> {
        let start_pos = self.pos;
        let mut num_str = String::new();

        // Optional negative sign
        if self.peek() == '-' {
            num_str.push(self.advance());
        }

        // Integer part
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.advance());
        }

        // Check for decimal point
        if !self.is_at_end()
            && self.peek() == '.'
            && self.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            num_str.push(self.advance()); // '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num_str.push(self.advance());
            }
            let len = self.pos - start_pos;
            let value: f64 = num_str.parse().map_err(|_| {
                PastelError::new(
                    ErrorKind::InvalidNumber,
                    format!("invalid float literal '{}'", num_str),
                )
                .with_span(self.current_span(len))
            })?;
            return Ok(Token {
                kind: TokenKind::Float(value),
                span: self.current_span(len),
            });
        }

        let len = self.pos - start_pos;
        let value: i64 = num_str.parse().map_err(|_| {
            PastelError::new(
                ErrorKind::InvalidNumber,
                format!("invalid integer literal '{}'", num_str),
            )
            .with_span(self.current_span(len))
        })?;
        Ok(Token {
            kind: TokenKind::Integer(value),
            span: self.current_span(len),
        })
    }

    fn scan_identifier(&mut self) -> Result<Token, PastelError> {
        let start_pos = self.pos;
        let mut ident = String::new();

        while !self.is_at_end()
            && (self.peek().is_ascii_alphanumeric() || self.peek() == '_' || self.peek() == '-')
        {
            ident.push(self.advance());
        }

        // Dot access: if followed by `.` and then an alpha/underscore, absorb the
        // dot and the next segment to form e.g. "colors.primary". Only do this
        // when the first segment is NOT a keyword (keywords never use dot access).
        if TokenKind::keyword(&ident).is_none() {
            while !self.is_at_end()
                && self.peek() == '.'
                && self
                    .peek_next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            {
                ident.push(self.advance()); // '.'
                while !self.is_at_end()
                    && (self.peek().is_ascii_alphanumeric()
                        || self.peek() == '_'
                        || self.peek() == '-')
                {
                    ident.push(self.advance());
                }
            }
        }

        let len = self.pos - start_pos;
        let kind = TokenKind::keyword(&ident).unwrap_or(TokenKind::Ident(ident));
        Ok(Token {
            kind,
            span: self.current_span(len),
        })
    }
}
