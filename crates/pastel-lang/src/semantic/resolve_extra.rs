use crate::ast::Expression;
use crate::error::{ErrorKind, PastelError};
use crate::ir::style::*;

use super::resolve::PropertyResolver;

/// Non-fill resolution helpers (split for file-size discipline).
impl<'a> PropertyResolver<'a> {
    pub(crate) fn is_number(&self, expr: &Expression) -> bool {
        let r = self.vars.resolve(expr);
        matches!(r, Expression::Integer(_) | Expression::Float(_))
    }

    pub fn resolve_bool(&self, expr: &Expression) -> Result<bool, PastelError> {
        let r = self.vars.resolve(expr);
        match r {
            Expression::Bool(b) => Ok(b),
            Expression::Ident(s) => match s.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(PastelError::new(
                    ErrorKind::TypeMismatch,
                    format!("expected bool, got '{s}'"),
                )),
            },
            _ => Err(PastelError::new(
                ErrorKind::TypeMismatch,
                format!("expected bool, got {:?}", r),
            )),
        }
    }

    pub fn resolve_position_mode(&self, expr: &Expression) -> Result<PositionMode, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "absolute" => Ok(PositionMode::Absolute),
            "relative" => Ok(PositionMode::Relative),
            _ => Err(PastelError::new(
                ErrorKind::InvalidValue,
                format!("unknown position mode '{s}'"),
            )
            .with_hint("expected: absolute, relative")),
        }
    }

    pub fn resolve_text_decoration(
        &self,
        expr: &Expression,
    ) -> Result<TextDecoration, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "none" => Ok(TextDecoration::None),
            "underline" => Ok(TextDecoration::Underline),
            "strikethrough" | "line-through" => Ok(TextDecoration::Strikethrough),
            _ => Err(PastelError::new(
                ErrorKind::InvalidValue,
                format!("unknown text decoration '{s}'"),
            )
            .with_hint("expected: none, underline, strikethrough")),
        }
    }

    pub fn resolve_text_transform(&self, expr: &Expression) -> Result<TextTransform, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "none" => Ok(TextTransform::None),
            "uppercase" => Ok(TextTransform::Uppercase),
            "lowercase" => Ok(TextTransform::Lowercase),
            _ => Err(PastelError::new(
                ErrorKind::InvalidValue,
                format!("unknown text transform '{s}'"),
            )
            .with_hint("expected: none, uppercase, lowercase")),
        }
    }

    pub fn resolve_stroke_dash(&self, expr: &Expression) -> Result<[f64; 2], PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Array(items) if items.len() == 2 => {
                Ok([self.resolve_f64(&items[0])?, self.resolve_f64(&items[1])?])
            }
            _ => Err(PastelError::new(
                ErrorKind::TypeMismatch,
                "expected stroke-dash as [dash, gap]",
            )
            .with_hint("e.g. stroke-dash = [8, 4]")),
        }
    }

    pub fn resolve_blend_mode(&self, expr: &Expression) -> Result<BlendMode, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "normal" => Ok(BlendMode::Normal),
            "multiply" => Ok(BlendMode::Multiply),
            "screen" => Ok(BlendMode::Screen),
            "overlay" => Ok(BlendMode::Overlay),
            "darken" => Ok(BlendMode::Darken),
            "lighten" => Ok(BlendMode::Lighten),
            _ => Err(PastelError::new(
                ErrorKind::InvalidValue,
                format!("unknown blend mode '{s}'"),
            )
            .with_hint("expected: normal, multiply, screen, overlay, darken, lighten")),
        }
    }

    pub fn resolve_justify(&self, expr: &Expression) -> Result<Justify, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "start" => Ok(Justify::Start),
            "center" => Ok(Justify::Center),
            "end" => Ok(Justify::End),
            "space-between" => Ok(Justify::SpaceBetween),
            "space-around" => Ok(Justify::SpaceAround),
            _ => Err(PastelError::new(
                ErrorKind::InvalidValue,
                format!("unknown justify '{s}'"),
            )),
        }
    }

    pub fn resolve_image_fit(&self, expr: &Expression) -> Result<ImageFit, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "cover" => Ok(ImageFit::Cover),
            "contain" => Ok(ImageFit::Contain),
            "fill" => Ok(ImageFit::Fill),
            "none" => Ok(ImageFit::None),
            _ => Err(PastelError::new(
                ErrorKind::InvalidValue,
                format!("unknown image fit '{s}'"),
            )),
        }
    }
}
