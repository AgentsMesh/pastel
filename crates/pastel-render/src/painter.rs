use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::{Color, Fill};
use pastel_lang::ir::IrDocument;
use skia_safe::{Canvas, Paint, Color4f, Rect, RRect, TextBlob};
use skia_safe::gradient_shader::GradientShaderColors;

use crate::layout::{LayoutTree, make_font};
use crate::painter_text::paint_text;

/// Paint the entire document onto a canvas.
pub fn paint_document(canvas: &Canvas, doc: &IrDocument, layout: &LayoutTree) {
    let bg = doc.canvas.background.as_ref()
        .map(color_to_skia)
        .unwrap_or(Color4f::new(1.0, 1.0, 1.0, 1.0));
    canvas.clear(bg);

    for node in &doc.nodes {
        paint_node(canvas, node, layout);
    }
}

fn paint_node(canvas: &Canvas, node: &IrNode, layout: &LayoutTree) {
    let rect = match layout.get(&node.id) {
        Some(r) => *r,
        None => return,
    };

    match &node.data {
        IrNodeData::Frame(f) => paint_frame(canvas, node, f, rect, layout),
        IrNodeData::Text(t) => paint_text(canvas, t, rect),
        IrNodeData::Image(img) => paint_image(canvas, img, rect),
        IrNodeData::Shape(s) => paint_shape(canvas, s, rect),
    }
}

fn paint_frame(
    canvas: &Canvas, node: &IrNode,
    f: &pastel_lang::ir::node::FrameData,
    rect: crate::layout::Rect, layout: &LayoutTree,
) {
    let sk_rect = to_sk_rect(rect);
    let cr = f.visual.corner_radius.as_ref().map(|r| r.0.map(|v| v as f32));
    let rrect = cr.map(|r| RRect::new_rect_radii(sk_rect, &corner_radii(r, rect.w, rect.h)));

    if let Some(shadow) = &f.visual.shadow {
        paint_shadow(canvas, shadow, rect, cr);
    }

    if let Some(fill) = &f.visual.fill {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        match fill {
            Fill::Solid { color } => { paint.set_color4f(color_to_skia(color), None); }
            Fill::LinearGradient { angle, stops } => {
                if let Some(shader) = make_gradient_shader(*angle, stops, sk_rect) {
                    paint.set_shader(shader);
                }
            }
            Fill::Transparent => {
                for child in &node.children { paint_node(canvas, child, layout); }
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
        if let Some(ref rr) = rrect { canvas.draw_rrect(rr, &paint); }
        else { canvas.draw_rect(sk_rect, &paint); }
    }

    for child in &node.children { paint_node(canvas, child, layout); }
}

fn paint_shadow(
    canvas: &Canvas, shadow: &pastel_lang::ir::style::Shadow,
    rect: crate::layout::Rect, cr: Option<[f32; 4]>,
) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(color_to_skia(&shadow.color), None);
    let blur = shadow.blur as f32;
    if blur > 0.0 {
        paint.set_mask_filter(skia_safe::MaskFilter::blur(
            skia_safe::BlurStyle::Normal, blur / 2.0, false,
        ));
    }
    let r = Rect::from_xywh(
        rect.x + shadow.x as f32, rect.y + shadow.y as f32, rect.w, rect.h,
    );
    if let Some(radii) = cr {
        canvas.draw_rrect(RRect::new_rect_radii(r, &corner_radii(radii, rect.w, rect.h)), &paint);
    } else {
        canvas.draw_rect(r, &paint);
    }
}

fn paint_image(
    canvas: &Canvas, img: &pastel_lang::ir::node::ImageData, rect: crate::layout::Rect,
) {
    let sk_rect = to_sk_rect(rect);
    let mut fill_paint = Paint::default();
    fill_paint.set_color4f(Color4f::new(0.91, 0.91, 0.91, 1.0), None);
    fill_paint.set_anti_alias(true);
    fill_paint.set_style(skia_safe::PaintStyle::Fill);

    let cr = img.corner_radius.as_ref().map(|r| r.0.map(|v| v as f32));
    if let Some(r) = cr {
        canvas.draw_rrect(RRect::new_rect_radii(sk_rect, &corner_radii(r, rect.w, rect.h)), &fill_paint);
    } else { canvas.draw_rect(sk_rect, &fill_paint); }

    let mut sp = Paint::default();
    sp.set_color4f(Color4f::new(0.82, 0.82, 0.82, 1.0), None);
    sp.set_style(skia_safe::PaintStyle::Stroke);
    sp.set_stroke_width(1.0);
    canvas.draw_rect(sk_rect, &sp);

    let label = img.name.as_deref().unwrap_or("image");
    let font = make_font(None, &None, 13.0);
    let mut tp = Paint::default();
    tp.set_color4f(Color4f::new(0.65, 0.65, 0.65, 1.0), None);
    let (tw, _) = font.measure_str(label, None);
    if let Some(blob) = TextBlob::from_str(label, &font) {
        canvas.draw_text_blob(&blob, (rect.x + (rect.w - tw) / 2.0, rect.y + rect.h / 2.0 + 4.0), &tp);
    }
}

fn paint_shape(
    canvas: &Canvas, s: &pastel_lang::ir::node::ShapeData, rect: crate::layout::Rect,
) {
    let sk_rect = to_sk_rect(rect);
    if let Some(fill) = &s.visual.fill {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        match fill {
            Fill::Solid { color } => { paint.set_color4f(color_to_skia(color), None); }
            Fill::LinearGradient { angle, stops } => {
                if let Some(shader) = make_gradient_shader(*angle, stops, sk_rect) {
                    paint.set_shader(shader);
                }
            }
            Fill::Transparent => {}
        };
        paint.set_style(skia_safe::PaintStyle::Fill);
        match s.shape_type {
            pastel_lang::ir::node::ShapeType::Ellipse => canvas.draw_oval(sk_rect, &paint),
            _ => canvas.draw_rect(sk_rect, &paint),
        };
    }
}

// -- Gradient --

fn make_gradient_shader(
    angle: f64, stops: &[pastel_lang::ir::style::GradientStop], rect: Rect,
) -> Option<skia_safe::Shader> {
    use skia_safe::{Point, gradient_shader, TileMode};
    use std::f64::consts::PI;

    let rad = angle * PI / 180.0;
    let cx = rect.x() + rect.width() / 2.0;
    let cy = rect.y() + rect.height() / 2.0;
    let (hw, hh) = (rect.width() / 2.0, rect.height() / 2.0);
    let (dx, dy) = (rad.sin() as f32, -(rad.cos()) as f32);
    let start = Point::new(cx - dx * hw, cy - dy * hh);
    let end = Point::new(cx + dx * hw, cy + dy * hh);

    let colors: Vec<Color4f> = stops.iter().map(|s| color_to_skia(&s.color)).collect();
    let pos: Vec<f32> = stops.iter().map(|s| (s.position / 100.0) as f32).collect();

    gradient_shader::linear(
        (start, end), GradientShaderColors::ColorsInSpace(&colors, None),
        pos.as_slice(), TileMode::Clamp, None, None,
    )
}

// -- Helpers --

pub(crate) fn color_to_skia(c: &Color) -> Color4f {
    Color4f::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0, c.a as f32 / 255.0)
}

fn to_sk_rect(r: crate::layout::Rect) -> Rect {
    Rect::from_xywh(r.x, r.y, r.w, r.h)
}

fn corner_radii(r: [f32; 4], w: f32, h: f32) -> [skia_safe::Point; 4] {
    let mr = (w.min(h)) / 2.0;
    [
        skia_safe::Point::new(r[0].min(mr), r[0].min(mr)),
        skia_safe::Point::new(r[1].min(mr), r[1].min(mr)),
        skia_safe::Point::new(r[2].min(mr), r[2].min(mr)),
        skia_safe::Point::new(r[3].min(mr), r[3].min(mr)),
    ]
}
