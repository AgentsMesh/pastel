use std::collections::HashMap;

use crate::ast::{ComponentDecl, NodeDecl, NodeKind};
use crate::error::PastelError;
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

    pub(super) fn props(&self) -> PropertyResolver<'_> {
        PropertyResolver::new(&self.vars)
    }

    pub(super) fn gen_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}_{}", prefix, self.id_counter)
    }

    pub fn build_node(&mut self, node: &NodeDecl) -> Result<IrNode, PastelError> {
        // Handle component instantiation (use)
        if node.kind == NodeKind::Use {
            return self.expand_component(node);
        }

        let id = if node.kind == NodeKind::Image {
            // Image nodes always get unique IDs to avoid layout collisions
            // when the same asset is referenced multiple times
            self.gen_id(&node.name.clone().unwrap_or_else(|| "image".into()))
        } else {
            node.name.clone().unwrap_or_else(|| {
                self.gen_id(match node.kind {
                    NodeKind::Frame => "frame",
                    NodeKind::Text => "text",
                    NodeKind::Image => unreachable!(),
                    NodeKind::Shape => "shape",
                    NodeKind::Use => unreachable!(),
                })
            })
        };

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
            padding: None, layout: None, position: None, rotation: None,
            visual: VisualProps::default(),
        };
        let (mut mode, mut gap, mut align, mut justify) = (None, None, None, None);
        let (mut columns, mut rows): (Option<u32>, Option<u32>) = (None, None);
        let (mut pos_mode, mut pos_top, mut pos_right, mut pos_bottom, mut pos_left) =
            (None, None, None, None, None);

        for attr in &node.attrs {
            match attr.key.as_str() {
                "width" => f.width = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "height" => f.height = Some(p.resolve_dimension(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "padding" => f.padding = Some(p.resolve_padding(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "fill" => f.visual.fill = Some(p.resolve_fill(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "stroke" => f.visual.stroke = Some(p.resolve_stroke(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "stroke-dash" => {
                    let dash = p.resolve_stroke_dash(&attr.value).map_err(|e| e.with_span(attr.span))?;
                    if let Some(ref mut s) = f.visual.stroke { s.dash = Some(dash); }
                }
                "radius" => f.visual.corner_radius = Some(p.resolve_corners(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "shadow" => f.visual.shadow = Some(p.resolve_shadow(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "inner-shadow" => f.visual.inner_shadow = Some(p.resolve_shadow(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "opacity" => f.visual.opacity = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "blur" => f.visual.blur = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "background-blur" => f.visual.background_blur = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "blend" => f.visual.blend = Some(p.resolve_blend_mode(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "rotation" => f.rotation = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "layout" => mode = Some(p.resolve_layout_mode(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "gap" => gap = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "align" => align = Some(p.resolve_align(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "justify" => justify = Some(p.resolve_justify(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "columns" => columns = Some(p.resolve_u32(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "rows" => rows = Some(p.resolve_u32(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "position" => pos_mode = Some(p.resolve_position_mode(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "top" => pos_top = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "right" => pos_right = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "bottom" => pos_bottom = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                "left" => pos_left = Some(p.resolve_f64(&attr.value).map_err(|e| e.with_span(attr.span))?),
                _ => {}
            }
        }

        if mode.is_some() || gap.is_some() || align.is_some() || justify.is_some() {
            f.layout = Some(Layout {
                mode: mode.unwrap_or(LayoutMode::Vertical), gap, align, justify,
                columns, rows,
            });
        }

        if pos_mode.is_some() || pos_top.is_some() || pos_right.is_some()
            || pos_bottom.is_some() || pos_left.is_some()
        {
            f.position = Some(Position {
                mode: pos_mode.unwrap_or(PositionMode::Relative),
                top: pos_top, right: pos_right, bottom: pos_bottom, left: pos_left,
            });
        }

        Ok(f)
    }
}
