use crate::ast::{Attribute, NodeDecl, NodeKind};
use crate::error::PastelError;
use crate::token::TokenKind;

use super::Parser;

impl Parser {
    /// Parse a node: frame/text/image/shape/use
    pub(super) fn parse_node(&mut self) -> Result<NodeDecl, PastelError> {
        let tok = self.advance().clone();
        let kind = match &tok.kind {
            TokenKind::Frame => NodeKind::Frame,
            TokenKind::Text => NodeKind::Text,
            TokenKind::Image => NodeKind::Image,
            TokenKind::Shape => NodeKind::Shape,
            TokenKind::Use => return self.parse_use_node(tok.span),
            _ => unreachable!("parse_node called with non-node token"),
        };

        let mut name = None;
        let mut label = None;

        if kind == NodeKind::Text {
            if let TokenKind::String(s) = &self.peek().kind {
                label = Some(s.clone());
                self.pos += 1;
            }
            if let TokenKind::Ident(s) = &self.peek().kind {
                name = Some(s.clone());
                self.pos += 1;
            }
        } else {
            if let TokenKind::Ident(s) = &self.peek().kind {
                name = Some(s.clone());
                self.pos += 1;
            }
        }

        self.expect(TokenKind::LBrace)?;
        let mut attrs = Vec::new();
        let mut children = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            match &self.peek().kind {
                TokenKind::Frame | TokenKind::Text | TokenKind::Image
                | TokenKind::Shape | TokenKind::Use => {
                    children.push(self.parse_node()?);
                }
                TokenKind::Ident(_) => {
                    attrs.push(self.parse_attribute()?);
                    self.match_token(&TokenKind::Comma);
                }
                _ => {
                    let tok = self.peek().clone();
                    return Err(crate::error::PastelError::new(
                        crate::error::ErrorKind::UnexpectedToken,
                        format!("unexpected token {:?} in node body", tok.kind),
                    )
                    .with_span(tok.span));
                }
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(NodeDecl { kind, name, label, attrs, children, span: tok.span })
    }

    /// Parse: use button("Sign Up", color = #333)
    fn parse_use_node(&mut self, span: crate::token::Span) -> Result<NodeDecl, PastelError> {
        let component_name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;

        // Parse args: positional first, then named (key = value)
        let mut attrs = Vec::new();
        let mut label = None;
        let mut arg_idx = 0;

        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            // Check if it's a named arg: ident = value
            if let TokenKind::Ident(_) = &self.peek().kind {
                let saved_pos = self.pos;
                let ident = self.expect_ident()?;
                if self.match_token(&TokenKind::Equals) {
                    let value = self.parse_expression()?;
                    attrs.push(Attribute { key: ident, value, span });
                } else {
                    // Not a named arg, restore and parse as positional
                    self.pos = saved_pos;
                    let value = self.parse_expression()?;
                    attrs.push(Attribute {
                        key: format!("__arg_{}", arg_idx),
                        value,
                        span,
                    });
                    arg_idx += 1;
                }
            } else {
                let value = self.parse_expression()?;
                // First positional string becomes label
                if label.is_none() {
                    if let crate::ast::Expression::String(s) = &value {
                        label = Some(s.clone());
                    }
                }
                attrs.push(Attribute {
                    key: format!("__arg_{}", arg_idx),
                    value,
                    span,
                });
                arg_idx += 1;
            }
            self.match_token(&TokenKind::Comma);
        }
        self.expect(TokenKind::RParen)?;

        Ok(NodeDecl {
            kind: NodeKind::Use,
            name: Some(component_name),
            label,
            attrs,
            children: Vec::new(),
            span,
        })
    }

    pub(super) fn parse_attributes(&mut self) -> Result<Vec<Attribute>, PastelError> {
        let mut attrs = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            attrs.push(self.parse_attribute()?);
            self.match_token(&TokenKind::Comma);
        }
        Ok(attrs)
    }

    fn parse_attribute(&mut self) -> Result<Attribute, PastelError> {
        let name_tok = self.peek().clone();
        let key = self.expect_ident()?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        Ok(Attribute { key, value, span: name_tok.span })
    }
}
