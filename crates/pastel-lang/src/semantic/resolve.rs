use crate::ast::Expression;
use crate::error::{ErrorKind, PastelError};
use crate::ir::style::*;
use std::collections::HashMap;

/// Resolves variables by substituting identifiers with their bound values.
pub struct VariableResolver {
    variables: HashMap<String, Expression>,
}

impl VariableResolver {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, value: Expression) {
        self.variables.insert(name, value);
    }

    pub fn resolve(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Ident(name) => {
                if let Some(resolved) = self.variables.get(name) {
                    self.resolve(resolved)
                } else {
                    expr.clone()
                }
            }
            Expression::FunctionCall { name, args } => {
                let resolved_args: Vec<Expression> = args.iter().map(|a| self.resolve(a)).collect();
                Expression::FunctionCall { name: name.clone(), args: resolved_args }
            }
            _ => expr.clone(),
        }
    }
}

/// Converts resolved expressions into typed IR values.
pub struct PropertyResolver<'a> {
    pub(crate) vars: &'a VariableResolver,
}

impl<'a> PropertyResolver<'a> {
    pub fn new(vars: &'a VariableResolver) -> Self {
        Self { vars }
    }

    pub fn resolve_f64(&self, expr: &Expression) -> Result<f64, PastelError> {
        let r = self.vars.resolve(expr);
        match r {
            Expression::Integer(n) => Ok(n as f64),
            Expression::Float(n) => Ok(n),
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected number, got {:?}", r))),
        }
    }

    pub fn resolve_u32(&self, expr: &Expression) -> Result<u32, PastelError> {
        let r = self.vars.resolve(expr);
        match r {
            Expression::Integer(n) if n >= 0 => Ok(n as u32),
            Expression::Float(n) if n >= 0.0 => Ok(n as u32),
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected positive integer, got {:?}", r))),
        }
    }

    pub fn resolve_string(&self, expr: &Expression) -> Result<String, PastelError> {
        let r = self.vars.resolve(expr);
        match r {
            Expression::String(s) => Ok(s),
            Expression::Ident(s) => Ok(s),
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected string, got {:?}", r))),
        }
    }

    pub fn resolve_color(&self, expr: &Expression) -> Result<Color, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Color(hex) => Color::from_hex(hex)
                .ok_or_else(|| PastelError::new(ErrorKind::InvalidValue, format!("invalid color: #{hex}"))),
            Expression::Ident(s) if s == "transparent" => Ok(Color::transparent()),
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected color, got {:?}", r))),
        }
    }

    pub fn resolve_dimension(&self, expr: &Expression) -> Result<Dimension, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Integer(n) => Ok(Dimension::Fixed(*n as f64)),
            Expression::Float(n) => Ok(Dimension::Fixed(*n)),
            Expression::Ident(s) => match s.as_str() {
                "fill" => Ok(Dimension::Fill),
                "hug" => Ok(Dimension::Hug),
                _ => Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown dimension '{s}'"))
                    .with_hint("expected a number, 'fill', or 'hug'")),
            },
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, format!("expected dimension, got {:?}", r))),
        }
    }

    pub fn resolve_stroke(&self, expr: &Expression) -> Result<Stroke, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Array(items) if items.len() == 2 => {
                Ok(Stroke { width: self.resolve_f64(&items[0])?, color: self.resolve_color(&items[1])?, dash: None })
            }
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, "expected stroke as [width, color]")
                .with_hint("e.g. stroke = [1, #DDDDDD]")),
        }
    }

    pub fn resolve_padding(&self, expr: &Expression) -> Result<Padding, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Array(items) => match items.len() {
                1 => { let v = self.resolve_f64(&items[0])?; Ok(Padding([v, v, v, v])) }
                2 => { let v = self.resolve_f64(&items[0])?; let h = self.resolve_f64(&items[1])?; Ok(Padding([v, h, v, h])) }
                4 => {
                    let t = self.resolve_f64(&items[0])?; let r = self.resolve_f64(&items[1])?;
                    let b = self.resolve_f64(&items[2])?; let l = self.resolve_f64(&items[3])?;
                    Ok(Padding([t, r, b, l]))
                }
                _ => Err(PastelError::new(ErrorKind::InvalidValue, "padding must have 1, 2, or 4 values")),
            },
            Expression::Integer(_) | Expression::Float(_) => {
                let v = self.resolve_f64(&r)?; Ok(Padding([v, v, v, v]))
            }
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, "expected padding as number or array")),
        }
    }

    pub fn resolve_corners(&self, expr: &Expression) -> Result<CornerRadius, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Integer(_) | Expression::Float(_) => {
                let v = self.resolve_f64(&r)?; Ok(CornerRadius([v, v, v, v]))
            }
            Expression::Array(items) if items.len() == 4 => {
                Ok(CornerRadius([
                    self.resolve_f64(&items[0])?, self.resolve_f64(&items[1])?,
                    self.resolve_f64(&items[2])?, self.resolve_f64(&items[3])?,
                ]))
            }
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, "expected radius as number or [tl, tr, br, bl]")),
        }
    }

    pub fn resolve_shadow(&self, expr: &Expression) -> Result<Shadow, PastelError> {
        let r = self.vars.resolve(expr);
        match &r {
            Expression::Array(items) if items.len() == 4 => Ok(Shadow {
                x: self.resolve_f64(&items[0])?, y: self.resolve_f64(&items[1])?,
                blur: self.resolve_f64(&items[2])?, color: self.resolve_color(&items[3])?,
            }),
            _ => Err(PastelError::new(ErrorKind::TypeMismatch, "expected shadow as [x, y, blur, color]")
                .with_hint("e.g. shadow = [0, 2, 8, #00000012]")),
        }
    }

    pub fn resolve_font_weight(&self, expr: &Expression) -> Result<FontWeight, PastelError> {
        let s = self.resolve_string(expr)?;
        FontWeight::from_str(&s).ok_or_else(|| {
            PastelError::new(ErrorKind::InvalidValue, format!("unknown font weight '{s}'"))
                .with_hint("expected: thin, light, normal, medium, semibold, bold, extrabold, black")
        })
    }

    pub fn resolve_text_align(&self, expr: &Expression) -> Result<TextAlign, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "left" => Ok(TextAlign::Left), "center" => Ok(TextAlign::Center), "right" => Ok(TextAlign::Right),
            _ => Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown text align '{s}'"))),
        }
    }

    pub fn resolve_layout_mode(&self, expr: &Expression) -> Result<LayoutMode, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "horizontal" => Ok(LayoutMode::Horizontal),
            "vertical" => Ok(LayoutMode::Vertical),
            "grid" => Ok(LayoutMode::Grid),
            _ => Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown layout mode '{s}'"))),
        }
    }

    pub fn resolve_align(&self, expr: &Expression) -> Result<Align, PastelError> {
        let s = self.resolve_string(expr)?;
        match s.as_str() {
            "start" => Ok(Align::Start), "center" => Ok(Align::Center),
            "end" => Ok(Align::End), "stretch" => Ok(Align::Stretch),
            _ => Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown align '{s}'"))),
        }
    }
}
