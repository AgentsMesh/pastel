use crate::ast::*;
use crate::error::{ErrorKind, PastelError};
use crate::token::TokenKind;

use super::Parser;

/// Component and page parsing (split for file-size discipline).
impl Parser {
    /// Parse: component button(label, color = primary) { ... }
    pub(super) fn parse_component(&mut self) -> Result<ComponentDecl, PastelError> {
        let span = self.expect(TokenKind::Component)?.span;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;

        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let param_name = self.expect_ident()?;
            let default = if self.match_token(&TokenKind::Equals) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            params.push(ComponentParam {
                name: param_name,
                default,
            });
            self.match_token(&TokenKind::Comma);
        }
        self.expect(TokenKind::RParen)?;

        // Component body: { node }
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_node()?;
        self.expect(TokenKind::RBrace)?;

        Ok(ComponentDecl {
            name,
            params,
            body,
            span,
        })
    }

    /// Parse: page "name" { frame ... text ... }
    pub(super) fn parse_page(&mut self) -> Result<PageDecl, PastelError> {
        let span = self.expect(TokenKind::Page)?.span;
        let name = self.expect_string()?;
        self.expect(TokenKind::LBrace)?;

        let mut nodes = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            match &self.peek().kind {
                TokenKind::Frame | TokenKind::Text | TokenKind::Image
                | TokenKind::Shape | TokenKind::Use => {
                    nodes.push(self.parse_node()?);
                }
                _ => {
                    let tok = self.peek().clone();
                    return Err(PastelError::new(
                        ErrorKind::UnexpectedToken,
                        format!("unexpected token {:?} in page body", tok.kind),
                    )
                    .with_span(tok.span)
                    .with_hint("pages can only contain node declarations"));
                }
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(PageDecl { name, nodes, span })
    }
}
