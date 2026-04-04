mod frame;
mod expr;
mod decl;

use crate::ast::*;
use crate::error::{ErrorKind, PastelError};
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, PastelError> {
        let mut program = Program {
            canvas: None,
            assets: Vec::new(),
            variables: Vec::new(),
            includes: Vec::new(),
            components: Vec::new(),
            nodes: Vec::new(),
            pages: Vec::new(),
        };

        while !self.is_at_end() {
            match &self.peek().kind {
                TokenKind::Canvas => {
                    program.canvas = Some(self.parse_canvas()?);
                }
                TokenKind::Asset => {
                    program.assets.push(self.parse_asset()?);
                }
                TokenKind::Let => {
                    program.variables.push(self.parse_let()?);
                }
                TokenKind::Include => {
                    program.includes.push(self.parse_include()?);
                }
                TokenKind::Component => {
                    program.components.push(self.parse_component()?);
                }
                TokenKind::Page => {
                    program.pages.push(self.parse_page()?);
                }
                TokenKind::Frame | TokenKind::Text | TokenKind::Image | TokenKind::Shape | TokenKind::Use => {
                    program.nodes.push(self.parse_node()?);
                }
                TokenKind::Eof => break,
                _ => {
                    let tok = self.peek().clone();
                    return Err(PastelError::new(
                        ErrorKind::ExpectedDeclaration,
                        format!("expected top-level declaration, found {:?}", tok.kind),
                    )
                    .with_span(tok.span)
                    .with_hint(
                        "expected: canvas, asset, let, include, component, page, frame, text, image, shape, use",
                    ));
                }
            }
        }

        Ok(program)
    }

    fn parse_canvas(&mut self) -> Result<CanvasDecl, PastelError> {
        let span = self.expect(TokenKind::Canvas)?.span;
        let name = self.expect_string()?;
        self.expect(TokenKind::LBrace)?;
        let attrs = self.parse_attributes()?;
        self.expect(TokenKind::RBrace)?;
        Ok(CanvasDecl { name, attrs, span })
    }

    fn parse_asset(&mut self) -> Result<AssetDecl, PastelError> {
        let span = self.expect(TokenKind::Asset)?.span;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Equals)?;
        let kind = self.expect_ident_or_keyword()?;
        self.expect(TokenKind::LParen)?;
        let path = self.expect_string()?;
        self.expect(TokenKind::RParen)?;
        Ok(AssetDecl { name, kind, path, span })
    }

    fn parse_let(&mut self) -> Result<LetDecl, PastelError> {
        let span = self.expect(TokenKind::Let)?.span;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        Ok(LetDecl { name, value, span })
    }

    fn parse_include(&mut self) -> Result<IncludeDecl, PastelError> {
        let span = self.expect(TokenKind::Include)?.span;
        let path = self.expect_string()?;
        Ok(IncludeDecl { path, span })
    }

    // -- Helpers --

    pub(crate) fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.peek().kind == TokenKind::Eof
    }

    pub(crate) fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub(crate) fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    pub(crate) fn expect(&mut self, expected: TokenKind) -> Result<Token, PastelError> {
        let tok = self.peek().clone();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(&expected) {
            self.pos += 1;
            Ok(tok)
        } else {
            Err(PastelError::new(
                ErrorKind::ExpectedToken,
                format!("expected {:?}, found {:?}", expected, tok.kind),
            )
            .with_span(tok.span))
        }
    }

    pub(crate) fn expect_string(&mut self) -> Result<String, PastelError> {
        let tok = self.peek().clone();
        if let TokenKind::String(s) = &tok.kind {
            let s = s.clone();
            self.pos += 1;
            Ok(s)
        } else {
            Err(PastelError::new(
                ErrorKind::ExpectedToken,
                format!("expected string literal, found {:?}", tok.kind),
            )
            .with_span(tok.span))
        }
    }

    pub(crate) fn expect_ident(&mut self) -> Result<String, PastelError> {
        let tok = self.peek().clone();
        if let TokenKind::Ident(s) = &tok.kind {
            let s = s.clone();
            self.pos += 1;
            Ok(s)
        } else {
            Err(PastelError::new(
                ErrorKind::ExpectedToken,
                format!("expected identifier, found {:?}", tok.kind),
            )
            .with_span(tok.span))
        }
    }

    /// Accept an identifier or a keyword token as a string.
    fn expect_ident_or_keyword(&mut self) -> Result<String, PastelError> {
        let tok = self.peek().clone();
        let name = match &tok.kind {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::Image => "image".into(),
            TokenKind::Text => "text".into(),
            TokenKind::Frame => "frame".into(),
            TokenKind::Shape => "shape".into(),
            TokenKind::Page => "page".into(),
            _ => {
                return Err(PastelError::new(
                    ErrorKind::ExpectedToken,
                    format!("expected identifier, found {:?}", tok.kind),
                )
                .with_span(tok.span));
            }
        };
        self.pos += 1;
        Ok(name)
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    pub(crate) fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}
