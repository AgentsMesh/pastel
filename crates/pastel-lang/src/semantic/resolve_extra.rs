use crate::ast::Expression;
use crate::error::{ErrorKind, PastelError};
use crate::ir::style::*;

use super::resolve::PropertyResolver;

/// Fill and gradient resolution logic, separated for file-size discipline.
impl<'a> PropertyResolver<'a> {
    pub fn resolve_fill(&self, expr: &Expression) -> Result<Fill, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Color(hex) => {
                let color = Color::from_hex(hex)
                    .ok_or_else(|| PastelError::new(ErrorKind::InvalidValue, format!("invalid color: #{hex}")))?;
                Ok(Fill::Solid { color })
            }
            Expression::Ident(s) if s == "transparent" => Ok(Fill::Transparent),
            Expression::FunctionCall { name, args } if name == "linear-gradient" => {
                self.resolve_linear_gradient(args)
            }
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected fill, got {:?}", r))),
        }
    }

    fn resolve_linear_gradient(&self, args: &[Expression]) -> Result<Fill, PastelError> {
        if args.len() < 3 {
            return Err(PastelError::new(
                ErrorKind::InvalidValue,
                "linear-gradient requires at least 3 arguments: angle, color1, color2",
            ).with_hint("e.g. linear-gradient(180, #FF0066, #3300FF)"));
        }

        let angle = self.resolve_f64(&args[0])?;
        let mut stops = Vec::new();
        let color_args = &args[1..];

        let mut i = 0;
        let color_count = self.count_gradient_colors(color_args);
        let mut color_idx = 0;

        while i < color_args.len() {
            let color = self.resolve_color(&color_args[i])?;
            i += 1;

            let position = if i < color_args.len() && self.is_number(&color_args[i]) {
                let pos = self.resolve_f64(&color_args[i])?;
                i += 1;
                pos
            } else {
                if color_count <= 1 { 0.0 }
                else { color_idx as f64 * 100.0 / (color_count - 1) as f64 }
            };

            stops.push(GradientStop { color, position });
            color_idx += 1;
        }

        Ok(Fill::LinearGradient { angle, stops })
    }

    fn count_gradient_colors(&self, args: &[Expression]) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < args.len() {
            let r = self.vars.resolve(&args[i]);
            match &r {
                Expression::Color(_) | Expression::Ident(_) => {
                    count += 1;
                    i += 1;
                    if i < args.len() && self.is_number(&args[i]) { i += 1; }
                }
                _ => { i += 1; }
            }
        }
        count
    }

    pub(crate) fn is_number(&self, expr: &Expression) -> bool {
        let r = self.vars.resolve(expr);
        matches!(r, Expression::Integer(_) | Expression::Float(_))
    }

    pub fn resolve_bool(&self, expr: &Expression) -> Result<bool, PastelError> {
        let r = self.vars.resolve(expr);
        match r {
            Expression::Bool(b) => Ok(b),
            Expression::Ident(s) => match s.as_str() {
                "true" => Ok(true), "false" => Ok(false),
                _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected bool, got '{s}'"))),
            },
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected bool, got {:?}", r))),
        }
    }

    pub fn resolve_position_mode(&self, expr: &Expression) -> Result<PositionMode, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "absolute" => Ok(PositionMode::Absolute),
            "relative" => Ok(PositionMode::Relative),
            _ => Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown position mode '{s}'"))
                .with_hint("expected: absolute, relative")),
        }
    }

    pub fn resolve_text_decoration(&self, expr: &Expression) -> Result<TextDecoration, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "none" => Ok(TextDecoration::None),
            "underline" => Ok(TextDecoration::Underline),
            "strikethrough" | "line-through" => Ok(TextDecoration::Strikethrough),
            _ => Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown text decoration '{s}'"))
                .with_hint("expected: none, underline, strikethrough")),
        }
    }

    pub fn resolve_text_transform(&self, expr: &Expression) -> Result<TextTransform, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "none" => Ok(TextTransform::None),
            "uppercase" => Ok(TextTransform::Uppercase),
            "lowercase" => Ok(TextTransform::Lowercase),
            _ => Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown text transform '{s}'"))
                .with_hint("expected: none, uppercase, lowercase")),
        }
    }
}
