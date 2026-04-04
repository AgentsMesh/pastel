use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::{Color, Fill};
use pastel_lang::ir::IrDocument;
use skia_safe::{Canvas, Paint, Color4f, Rect, RRect, TextBlob};

use crate::layout::{LayoutTree, make_font};

/// Paint the entire document onto a canvas.
pub fn paint_document(canvas: &Canvas, doc: &IrDocument, layout: &LayoutTree) {
    // Background
    let bg = doc.canvas.background.as_ref()
        .map(color_to_skia)
        .unwrap_or(skia_safe::Color4f::new(1.0, 1.0, 1.0, 1.0));
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
    let rrect = cr.map(|r| RRect::new_rect_radii(sk_rect, &corner_radii(r)));

    // Shadow
    if let Some(shadow) = &f.visual.shadow {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color4f(color_to_skia(&shadow.color), None);
        let blur = shadow.blur as f32;
        if blur > 0.0 {
            paint.set_mask_filter(skia_safe::MaskFilter::blur(
                skia_safe::BlurStyle::Normal, blur / 2.0, false,
            ));
        }
        let offset_rect = Rect::from_xywh(
            rect.x + shadow.x as f32, rect.y + shadow.y as f32, rect.w, rect.h,
        );
        if let Some(r) = &cr {
            canvas.draw_rrect(RRect::new_rect_radii(offset_rect, &corner_radii(*r)), &paint);
        } else {
            canvas.draw_rect(offset_rect, &paint);
        }
    }

    // Fill
    if let Some(fill) = &f.visual.fill {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        match fill {
            Fill::Solid { color } => paint.set_color4f(color_to_skia(color), None),
            Fill::Transparent => return, // nothing to draw
        };
        paint.set_style(skia_safe::PaintStyle::Fill);

        if let Some(ref rr) = rrect {
            canvas.draw_rrect(rr, &paint);
        } else {
            canvas.draw_rect(sk_rect, &paint);
        }
    }

    // Stroke
    if let Some(stroke) = &f.visual.stroke {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color4f(color_to_skia(&stroke.color), None);
        paint.set_style(skia_safe::PaintStyle::Stroke);
        paint.set_stroke_width(stroke.width as f32);

        if let Some(ref rr) = rrect {
            canvas.draw_rrect(rr, &paint);
        } else {
            canvas.draw_rect(sk_rect, &paint);
        }
    }

    // Children
    for child in &node.children {
        paint_node(canvas, child, layout);
    }
}

fn paint_text(canvas: &Canvas, t: &pastel_lang::ir::node::TextData, rect: crate::layout::Rect) {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let font = make_font(t.font_family.as_deref(), &t.font_weight, fs);

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let color = t.color.as_ref()
        .map(color_to_skia)
        .unwrap_or(Color4f::new(0.0, 0.0, 0.0, 1.0));
    paint.set_color4f(color, None);

    // Vertical center
    let metrics = font.metrics();
    let text_h = -metrics.1.ascent + metrics.1.descent;
    let ty = rect.y + (rect.h - text_h) / 2.0 - metrics.1.ascent;

    // Horizontal alignment
    let (text_w, _) = font.measure_str(&t.content, None);
    let tx = match t.text_align {
        Some(pastel_lang::ir::style::TextAlign::Center) => rect.x + (rect.w - text_w) / 2.0,
        Some(pastel_lang::ir::style::TextAlign::Right) => rect.x + rect.w - text_w,
        _ => rect.x,
    };

    if let Some(blob) = TextBlob::from_str(&t.content, &font) {
        canvas.draw_text_blob(&blob, (tx, ty), &paint);
    }
}

fn paint_image(
    canvas: &Canvas, img: &pastel_lang::ir::node::ImageData, rect: crate::layout::Rect,
) {
    let sk_rect = to_sk_rect(rect);

    // Placeholder
    let mut fill_paint = Paint::default();
    fill_paint.set_color4f(Color4f::new(0.91, 0.91, 0.91, 1.0), None);
    fill_paint.set_anti_alias(true);
    fill_paint.set_style(skia_safe::PaintStyle::Fill);

    let cr = img.corner_radius.as_ref().map(|r| r.0.map(|v| v as f32));
    if let Some(r) = cr {
        canvas.draw_rrect(RRect::new_rect_radii(sk_rect, &corner_radii(r)), &fill_paint);
    } else {
        canvas.draw_rect(sk_rect, &fill_paint);
    }

    // Border
    let mut stroke_paint = Paint::default();
    stroke_paint.set_color4f(Color4f::new(0.82, 0.82, 0.82, 1.0), None);
    stroke_paint.set_style(skia_safe::PaintStyle::Stroke);
    stroke_paint.set_stroke_width(1.0);
    canvas.draw_rect(sk_rect, &stroke_paint);

    // Label
    let label = img.name.as_deref().unwrap_or("image");
    let font = make_font(None, &None, 13.0);
    let mut text_paint = Paint::default();
    text_paint.set_color4f(Color4f::new(0.65, 0.65, 0.65, 1.0), None);
    let (tw, _) = font.measure_str(label, None);
    let tx = rect.x + (rect.w - tw) / 2.0;
    let ty = rect.y + rect.h / 2.0 + 4.0;
    if let Some(blob) = TextBlob::from_str(label, &font) {
        canvas.draw_text_blob(&blob, (tx, ty), &text_paint);
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
            Fill::Transparent => {}
        };
        paint.set_style(skia_safe::PaintStyle::Fill);

        match s.shape_type {
            pastel_lang::ir::node::ShapeType::Ellipse => {
                canvas.draw_oval(sk_rect, &paint);
            }
            _ => {
                canvas.draw_rect(sk_rect, &paint);
            }
        }
    }
}

// -- Helpers --

fn color_to_skia(c: &Color) -> Color4f {
    Color4f::new(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    )
}

fn to_sk_rect(r: crate::layout::Rect) -> Rect {
    Rect::from_xywh(r.x, r.y, r.w, r.h)
}

fn corner_radii(r: [f32; 4]) -> [skia_safe::Point; 4] {
    [
        skia_safe::Point::new(r[0], r[0]),
        skia_safe::Point::new(r[1], r[1]),
        skia_safe::Point::new(r[2], r[2]),
        skia_safe::Point::new(r[3], r[3]),
    ]
}
