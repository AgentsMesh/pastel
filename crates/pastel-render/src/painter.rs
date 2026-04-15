use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::{Color, Fill};
use pastel_lang::ir::IrDocument;
use skia_safe::{Canvas, Paint, Color4f, Rect, RRect};

use crate::layout::LayoutTree;
use crate::image_cache::ImageCache;
use crate::painter_text::paint_text;
use crate::painter_leaf::{paint_shadow, paint_image, make_gradient_shader, make_radial_gradient_shader};
use crate::painter_effects::{apply_rotation, apply_blend_mode, paint_inner_shadow, apply_blur_filter, apply_dash_effect};

/// Paint the entire document onto a canvas.
pub fn paint_document(canvas: &Canvas, doc: &IrDocument, layout: &LayoutTree, images: &ImageCache) {
    let bg = doc.canvas.background.as_ref()
        .map(color_to_skia)
        .unwrap_or(Color4f::new(1.0, 1.0, 1.0, 1.0));
    canvas.clear(bg);

    for node in &doc.nodes {
        paint_node(canvas, node, layout, images);
    }
}

/// Paint specific nodes onto a canvas (for page rendering).
pub fn paint_nodes(canvas: &Canvas, doc: &IrDocument, nodes: &[IrNode], layout: &LayoutTree, images: &ImageCache) {
    let bg = doc.canvas.background.as_ref()
        .map(color_to_skia)
        .unwrap_or(Color4f::new(1.0, 1.0, 1.0, 1.0));
    canvas.clear(bg);

    for node in nodes {
        paint_node(canvas, node, layout, images);
    }
}

fn paint_node(canvas: &Canvas, node: &IrNode, layout: &LayoutTree, images: &ImageCache) {
    let rect = match layout.get(&node.id) {
        Some(r) => *r,
        None => return,
    };
    match &node.data {
        IrNodeData::Frame(f) => paint_frame(canvas, node, f, rect, layout, images),
        IrNodeData::Text(t) => paint_text(canvas, t, rect),
        IrNodeData::Image(img) => paint_image(canvas, img, rect, images),
        IrNodeData::Shape(s) => paint_shape(canvas, s, rect),
    }
}

fn paint_frame(
    canvas: &Canvas, node: &IrNode,
    f: &pastel_lang::ir::node::FrameData,
    rect: crate::layout::Rect, layout: &LayoutTree, images: &ImageCache,
) {
    let sk_rect = to_sk_rect(rect);
    let cr = f.visual.corner_radius.as_ref().map(|r| r.0.map(|v| v as f32));
    let rrect = cr.map(|r| RRect::new_rect_radii(sk_rect, &corner_radii(r, rect.w, rect.h)));

    let has_rotation = apply_rotation(canvas, f.rotation, rect);
    let has_blend = apply_blend_mode(canvas, &f.visual);

    if let Some(shadow) = &f.visual.shadow {
        paint_shadow(canvas, shadow, rect, cr);
    }

    if let Some(fill) = &f.visual.fill {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        apply_blur_filter(&mut paint, &f.visual);
        match fill {
            Fill::Solid { color } => { paint.set_color4f(color_to_skia(color), None); }
            Fill::LinearGradient { angle, stops } => {
                if let Some(shader) = make_gradient_shader(*angle, stops, sk_rect) {
                    paint.set_shader(shader);
                }
            }
            Fill::RadialGradient { cx, cy, stops } => {
                if let Some(shader) = make_radial_gradient_shader(*cx, *cy, stops, sk_rect) {
                    paint.set_shader(shader);
                }
            }
            Fill::Transparent => {
                for child in &node.children { paint_node(canvas, child, layout, images); }
                if has_blend { canvas.restore(); }
                if has_rotation { canvas.restore(); }
                return;
            }
        };
        paint.set_style(skia_safe::PaintStyle::Fill);
        if let Some(ref rr) = rrect { canvas.draw_rrect(rr, &paint); }
        else { canvas.draw_rect(sk_rect, &paint); }
    }

    if let Some(stroke) = &f.visual.stroke {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color4f(color_to_skia(&stroke.color), None);
        paint.set_style(skia_safe::PaintStyle::Stroke);
        paint.set_stroke_width(stroke.width as f32);
        apply_dash_effect(&mut paint, stroke);
        if let Some(ref rr) = rrect { canvas.draw_rrect(rr, &paint); }
        else { canvas.draw_rect(sk_rect, &paint); }
    }

    if let Some(inner_shadow) = &f.visual.inner_shadow {
        paint_inner_shadow(canvas, inner_shadow, rect, cr);
    }

    for child in &node.children { paint_node(canvas, child, layout, images); }

    if has_blend { canvas.restore(); }
    if has_rotation { canvas.restore(); }
}

fn paint_shape(
    canvas: &Canvas, s: &pastel_lang::ir::node::ShapeData, rect: crate::layout::Rect,
) {
    use pastel_lang::ir::node::ShapeType;

    let sk_rect = to_sk_rect(rect);
    let has_rotation = apply_rotation(canvas, s.rotation, rect);
    let has_blend = apply_blend_mode(canvas, &s.visual);

    let svg_path = s.path.as_ref().and_then(|d| {
        skia_safe::utils::parse_path::from_svg(d).map(|p| {
            let bounds = p.bounds();
            let view_w = s.width.as_ref().and_then(|d| match d {
                pastel_lang::ir::style::Dimension::Fixed(n) => Some(*n as f32), _ => None,
            }).unwrap_or(bounds.width());
            let view_h = s.height.as_ref().and_then(|d| match d {
                pastel_lang::ir::style::Dimension::Fixed(n) => Some(*n as f32), _ => None,
            }).unwrap_or(bounds.height());

            let scale_x = if view_w > 0.0 { rect.w / view_w } else { 1.0 };
            let scale_y = if view_h > 0.0 { rect.h / view_h } else { 1.0 };

            let mut m = skia_safe::Matrix::new_identity();
            m.pre_translate((rect.x, rect.y));
            m.pre_scale((scale_x, scale_y), None);
            p.with_transform(&m)
        })
    });

    if let Some(fill) = &s.visual.fill {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        apply_blur_filter(&mut paint, &s.visual);
        match fill {
            Fill::Solid { color } => { paint.set_color4f(color_to_skia(color), None); }
            Fill::LinearGradient { angle, stops } => {
                if let Some(shader) = make_gradient_shader(*angle, stops, sk_rect) { paint.set_shader(shader); }
            }
            Fill::RadialGradient { cx, cy, stops } => {
                if let Some(shader) = make_radial_gradient_shader(*cx, *cy, stops, sk_rect) { paint.set_shader(shader); }
            }
            Fill::Transparent => {}
        };
        paint.set_style(skia_safe::PaintStyle::Fill);
        if let Some(ref path) = svg_path {
            canvas.draw_path(path, &paint);
        } else {
            match s.shape_type {
                ShapeType::Ellipse => canvas.draw_oval(sk_rect, &paint),
                _ => canvas.draw_rect(sk_rect, &paint),
            };
        }
    }

    if let Some(stroke) = &s.visual.stroke {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color4f(color_to_skia(&stroke.color), None);
        paint.set_style(skia_safe::PaintStyle::Stroke);
        paint.set_stroke_width(stroke.width as f32);
        apply_dash_effect(&mut paint, stroke);
        if let Some(ref path) = svg_path {
            canvas.draw_path(path, &paint);
        } else {
            match s.shape_type {
                ShapeType::Ellipse => canvas.draw_oval(sk_rect, &paint),
                _ => canvas.draw_rect(sk_rect, &paint),
            };
        }
    }

    if has_blend { canvas.restore(); }
    if has_rotation { canvas.restore(); }
}

// -- Helpers --

pub(crate) fn color_to_skia(c: &Color) -> Color4f {
    Color4f::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0, c.a as f32 / 255.0)
}

pub(crate) fn to_sk_rect(r: crate::layout::Rect) -> Rect {
    Rect::from_xywh(r.x, r.y, r.w, r.h)
}

pub(crate) fn corner_radii(r: [f32; 4], w: f32, h: f32) -> [skia_safe::Point; 4] {
    let mr = (w.min(h)) / 2.0;
    [
        skia_safe::Point::new(r[0].min(mr), r[0].min(mr)),
        skia_safe::Point::new(r[1].min(mr), r[1].min(mr)),
        skia_safe::Point::new(r[2].min(mr), r[2].min(mr)),
        skia_safe::Point::new(r[3].min(mr), r[3].min(mr)),
    ]
}
