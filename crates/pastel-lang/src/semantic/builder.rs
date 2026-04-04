use std::collections::HashMap;

use crate::ast::{ComponentDecl, NodeDecl, NodeKind};
use crate::error::{ErrorKind, PastelError};
use crate::ir::node::*;
use crate::ir::style::*;
use crate::ir::IrAsset;

use super::resolve::{PropertyResolver, VariableResolver};

/// Builds the IR tree from AST nodes using resolved properties.
pub struct IrBuilder {
    pub(super) vars: VariableResolver,
    #[allow(dead_code)]
    pub(super) assets: HashMap<String, IrAsset>,
    pub(super) components: HashMap<String, ComponentDecl>,
    id_counter: usize,
}

impl IrBuilder {
    pub fn new(
        vars: VariableResolver,
        assets: HashMap<String, IrAsset>,
        components: Vec<ComponentDecl>,
    ) -> Self {
        let comp_map = components.into_iter().map(|c| (c.name.clone(), c)).collect();
        Self { vars, assets, components: comp_map, id_counter: 0 }
    }

    fn props(&self) -> PropertyResolver<'_> {
        PropertyResolver::new(&self.vars)
    }

    fn gen_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}_{}", prefix, self.id_counter)
    }

    pub fn build_node(&mut self, node: &NodeDecl) -> Result<IrNode, PastelError> {
        // Handle component instantiation (use)
        if node.kind == NodeKind::Use {
            return self.expand_component(node);
        }

        let id = node.name.clone().unwrap_or_else(|| {
            self.gen_id(match node.kind {
                NodeKind::Frame => "frame",
                NodeKind::Text => "text",
                NodeKind::Image => "image",
                NodeKind::Shape => "shape",
                NodeKind::Use => unreachable!(),
            })
        });

        let children = node.children.iter()
            .map(|c| self.build_node(c))
            .collect::<Result<Vec<_>, _>>()?;

        let data = match node.kind {
            NodeKind::Frame => IrNodeData::Frame(self.build_frame(node)?),
            NodeKind::Text => IrNodeData::Text(self.build_text(node)?),
            NodeKind::Image => IrNodeData::Image(self.build_image(node)?),
            NodeKind::Shape => IrNodeData::Shape(self.build_shape(node)?),
            NodeKind::Use => unreachable!("Use nodes handled by expand_component"),
        };

        Ok(IrNode { id, data, children })
    }

    fn build_frame(&self, node: &NodeDecl) -> Result<FrameData, PastelError> {
        let p = self.props();
        let mut f = FrameData {
            name: node.name.clone(), width: None, height: None,
            padding: None, layout: None, visual: VisualProps::default(),
        };
        let (mut mode, mut gap, mut align, mut justify) = (None, None, None, None);

        for attr in &node.attrs {
            match attr.key.as_str() {
                "width" => f.width = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "height" => f.height = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "padding" => f.padding = Some(p.resolve_padding(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "fill" => f.visual.fill = Some(p.resolve_fill(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "stroke" => f.visual.stroke = Some(p.resolve_stroke(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "radius" => f.visual.corner_radius = Some(p.resolve_corners(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "shadow" => f.visual.shadow = Some(p.resolve_shadow(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "opacity" => f.visual.opacity = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "layout" => mode = Some(p.resolve_layout_mode(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "gap" => gap = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "align" => align = Some(p.resolve_align(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "justify" => justify = Some(p.resolve_justify(&attr.value).map_err(|e| e.with_span(attr.span))?),
                _ => {}
            }
        }

        if mode.is_some() || gap.is_some() || align.is_some() || justify.is_some() {
            f.layout = Some(Layout {
                mode: mode.unwrap_or(LayoutMode::Vertical), gap, align, justify,
            });
        }
        Ok(f)
    }

    fn build_text(&self, node: &NodeDecl) -> Result<TextData, PastelError> {
        let p = self.props();
        let mut t = TextData {
            content: node.label.clone().unwrap_or_default(),
            font_size: None, font_weight: None, font_family: None,
            color: None, text_align: None, line_height: None,
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
                _ => {}
            }
        }
        Ok(t)
    }

    fn build_image(&self, node: &NodeDecl) -> Result<ImageData, PastelError> {
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

    fn build_shape(&self, node: &NodeDecl) -> Result<ShapeData, PastelError> {
        let p = self.props();
        let mut s = ShapeData {
            name: node.name.clone(), shape_type: ShapeType::Rectangle,
            width: None, height: None, visual: VisualProps::default(),
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
                "radius" => s.visual.corner_radius = Some(p.resolve_corners(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "shadow" => s.visual.shadow = Some(p.resolve_shadow(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "opacity" => s.visual.opacity = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                _ => {}
            }
        }
        Ok(s)
    }
}
