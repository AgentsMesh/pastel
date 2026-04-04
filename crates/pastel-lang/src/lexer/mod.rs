mod scan;

use crate::error::PastelError;
use crate::token::{Span, Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, PastelError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    span: self.current_span(0),
                });
                break;
            }
            let token = self.scan_token()?;
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            match ch {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.col = 1;
                    continue; // col already reset
                }
                '/' if self.peek_next() == Some('/') => {
                    // Line comment: skip to end of line
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                    continue;
                }
                _ => break,
            };
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&self) -> char {
        self.source[self.pos]
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.pos];
        self.pos += 1;
        self.col += 1;
        ch
    }

    fn current_span(&self, len: usize) -> Span {
        Span {
            start: self.pos.saturating_sub(len),
            end: self.pos,
            line: self.line,
            col: self.col.saturating_sub(len),
        }
    }
}
