use crate::ast::Expression;
use crate::error::{ErrorKind, PastelError};
use crate::token::TokenKind;

use super::Parser;

impl Parser {
    /// Parse a value expression.
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, PastelError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.pos += 1;
                Ok(Expression::Integer(n))
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.pos += 1;
                Ok(Expression::Float(n))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.pos += 1;
                Ok(Expression::String(s))
            }
            TokenKind::Color(c) => {
                let c = c.clone();
                self.pos += 1;
                Ok(Expression::Color(c))
            }
            TokenKind::Bool(b) => {
                let b = *b;
                self.pos += 1;
                Ok(Expression::Bool(b))
            }
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.pos += 1;
                // Check for function call: ident(...)
                if self.check(&TokenKind::LParen) {
                    return self.parse_function_call(s);
                }
                Ok(Expression::Ident(s))
            }
            TokenKind::LBracket => self.parse_array(),
            TokenKind::LBrace => self.parse_object(),
            _ => Err(PastelError::new(
                ErrorKind::UnexpectedToken,
                format!("expected value expression, found {:?}", tok.kind),
            )
            .with_span(tok.span)),
        }
    }

    fn parse_array(&mut self) -> Result<Expression, PastelError> {
        self.expect(TokenKind::LBracket)?;
        let mut items = Vec::new();
        while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
            items.push(self.parse_expression()?);
            self.match_token(&TokenKind::Comma);
        }
        self.expect(TokenKind::RBracket)?;
        Ok(Expression::Array(items))
    }

    fn parse_function_call(&mut self, name: String) -> Result<Expression, PastelError> {
        self.expect(TokenKind::LParen)?;
        let mut args = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            args.push(self.parse_expression()?);
            self.match_token(&TokenKind::Comma);
        }
        self.expect(TokenKind::RParen)?;
        Ok(Expression::FunctionCall { name, args })
    }

    /// Parse: { key = value, key2 = value2, ... }
    fn parse_object(&mut self) -> Result<Expression, PastelError> {
        self.expect(TokenKind::LBrace)?;
        let mut entries = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let key = self.expect_ident()?;
            self.expect(TokenKind::Equals)?;
            let value = self.parse_expression()?;
            entries.push((key, value));
            self.match_token(&TokenKind::Comma);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Expression::Object(entries))
    }
}
