use crate::ast::NodeDecl;
use crate::error::{ErrorKind, PastelError};
use crate::ir::node::*;

use super::builder::IrBuilder;

/// Leaf-node builders: image, shape, text (split for file-size discipline).
impl IrBuilder {
    pub(super) fn build_text(&self, node: &NodeDecl) -> Result<TextData, PastelError> {
        let p = self.props();
        let mut t = TextData {
            content: node.label.clone().unwrap_or_default(),
            font_size: None, font_weight: None, font_family: None,
            color: None, text_align: None, line_height: None,
            width: None, height: None, wrap: None,
            letter_spacing: None, text_decoration: None, text_transform: None,
        };
        for attr in &node.attrs {
            match attr.key.as_str() {
                "content" => t.content = p.resolve_string(&attr.value).map_err(|e| e.with_span(attr.span))?,
                "size" => t.font_size = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "weight" => t.font_weight = Some(p.resolve_font_weight(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "font" => t.font_family = Some(p.resolve_string(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "color" => t.color = Some(p.resolve_color(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "align" => t.text_align = Some(p.resolve_text_align(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "line-height" => t.line_height = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "width" => t.width = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "height" => t.height = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "wrap" => t.wrap = Some(p.resolve_bool(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "letter-spacing" => t.letter_spacing = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "text-decoration" => t.text_decoration = Some(p.resolve_text_decoration(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "text-transform" => t.text_transform = Some(p.resolve_text_transform(&attr.value).map_err(|e| e.with_span(attr.span))?),
                _ => {}
            }
        }
        Ok(t)
    }

    pub(super) fn build_image(&self, node: &NodeDecl) -> Result<ImageData, PastelError> {
        let p = self.props();
        let mut img = ImageData {
            name: node.name.clone(), asset: node.name.clone().unwrap_or_default(),
            width: None, height: None, corner_radius: None, shadow: None, opacity: None, fit: None,
        };
        for attr in &node.attrs {
            match attr.key.as_str() {
                "width" => img.width = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "height" => img.height = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "radius" => img.corner_radius = Some(p.resolve_corners(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "shadow" => img.shadow = Some(p.resolve_shadow(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "opacity" => img.opacity = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "fit" => img.fit = Some(p.resolve_image_fit(&attr.value).map_err(|e| e.with_span(attr.span))?),
                _ => {}
            }
        }
        Ok(img)
    }

    pub(super) fn build_shape(&self, node: &NodeDecl) -> Result<ShapeData, PastelError> {
        let p = self.props();
        let mut s = ShapeData {
            name: node.name.clone(), shape_type: ShapeType::Rectangle,
            width: None, height: None, rotation: None, visual: VisualProps::default(),
        };
        for attr in &node.attrs {
            match attr.key.as_str() {
                "type" => {
                    let v = p.resolve_string(&attr.value).map_err(|e| e.with_span(attr.span))?;
                    s.shape_type = match v.as_str() {
                        "rectangle" | "rect" => ShapeType::Rectangle,
                        "ellipse" | "circle" => ShapeType::Ellipse,
                        "line" => ShapeType::Line,
                        _ => return Err(PastelError::new(ErrorKind::InvalidValue, format!("unknown shape type '{v}'"))
                            .with_span(attr.span).with_hint("expected: rectangle, ellipse, line")),
                    };
                }
                "width" => s.width = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "height" => s.height = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "fill" => s.visual.fill = Some(p.resolve_fill(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "stroke" => s.visual.stroke = Some(p.resolve_stroke(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "stroke-dash" => {
                    let dash = p.resolve_stroke_dash(&attr.value).map_err(|e| e.with_span(attr.span))?;
                    if let Some(ref mut st) = s.visual.stroke { st.dash = Some(dash); }
                }
                "radius" => s.visual.corner_radius = Some(p.resolve_corners(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "shadow" => s.visual.shadow = Some(p.resolve_shadow(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "inner-shadow" => s.visual.inner_shadow = Some(p.resolve_shadow(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "opacity" => s.visual.opacity = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "blur" => s.visual.blur = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "blend" => s.visual.blend = Some(p.resolve_blend_mode(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "rotation" => s.rotation = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                _ => {}
            }
        }
        Ok(s)
    }
}
