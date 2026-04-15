use crate::ast::Expression;
use crate::error::{ErrorKind, PastelError};
use crate::ir::style::*;

use super::resolve::PropertyResolver;

/// Fill and gradient resolution (split for file-size discipline).
impl<'a> PropertyResolver<'a> {
    pub fn resolve_fill(&self, expr: &Expression) -> Result<Fill, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Color(hex) => {
                let color = Color::from_hex(hex).ok_or_else(|| {
                    PastelError::new(ErrorKind::InvalidValue, format!("invalid color: #{hex}"))
                })?;
                Ok(Fill::Solid { color })
            }
            Expression::Ident(s) if s == "transparent" => Ok(Fill::Transparent),
            Expression::FunctionCall { name, args } if name == "linear-gradient" => {
                self.resolve_linear_gradient(args)
            }
            Expression::FunctionCall { name, args } if name == "radial-gradient" => {
                self.resolve_radial_gradient(args)
            }
            _ => Err(PastelError::new(
                ErrorKind::TypeMismatch,
                format!("expected fill, got {:?}", r),
            )),
        }
    }

    fn resolve_linear_gradient(&self, args: &[Expression]) -> Result<Fill, PastelError> {
        if args.len() < 3 {
            return Err(PastelError::new(
                ErrorKind::InvalidValue,
                "linear-gradient requires at least 3 arguments: angle, color1, color2",
            )
            .with_hint("e.g. linear-gradient(180, #FF0066, #3300FF)"));
        }

        let angle = self.resolve_f64(&args[0])?;
        let color_args = &args[1..];
        self.parse_gradient_stops(color_args)
            .map(|stops| Fill::LinearGradient { angle, stops })
    }

    fn resolve_radial_gradient(&self, args: &[Expression]) -> Result<Fill, PastelError> {
        if args.len() < 2 {
            return Err(PastelError::new(
                ErrorKind::InvalidValue,
                "radial-gradient requires at least 2 color arguments",
            )
            .with_hint("e.g. radial-gradient(#FF6B6B, #4ECDC4)"));
        }

        let (cx, cy, color_args) =
            if args.len() >= 4 && self.is_number(&args[0]) && self.is_number(&args[1]) {
                (
                    self.resolve_f64(&args[0])?,
                    self.resolve_f64(&args[1])?,
                    &args[2..],
                )
            } else {
                (50.0, 50.0, args)
            };

        self.parse_gradient_stops(color_args)
            .map(|stops| Fill::RadialGradient { cx, cy, stops })
    }

    fn parse_gradient_stops(&self, args: &[Expression]) -> Result<Vec<GradientStop>, PastelError> {
        let color_count = self.count_gradient_colors(args);
        let mut stops = Vec::new();
        let mut i = 0;
        let mut color_idx = 0;

        while i < args.len() {
            let color = self.resolve_color(&args[i])?;
            i += 1;

            let position = if i < args.len() && self.is_number(&args[i]) {
                let pos = self.resolve_f64(&args[i])?;
                i += 1;
                pos
            } else if color_count <= 1 {
                0.0
            } else {
                color_idx as f64 * 100.0 / (color_count - 1) as f64
            };

            stops.push(GradientStop { color, position });
            color_idx += 1;
        }
        Ok(stops)
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
                    if i < args.len() && self.is_number(&args[i]) {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
        count
    }
}
