use pastel_lang::ir::style::TextDecoration;
use skia_safe::{Canvas, Paint};

use crate::layout::{apply_text_transform, make_font, make_font_style};
use crate::text_shaping::{shape_text, wrap_shaped_lines, ShapedText};

/// Paint a text node (single-line or wrapped).
pub(crate) fn paint_text(
    canvas: &Canvas, t: &pastel_lang::ir::node::TextData, rect: crate::layout::Rect,
) {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let spacing = t.letter_spacing.unwrap_or(0.0) as f32;
    let font = make_font(t.font_family.as_deref(), &t.font_weight, fs);
    let style = make_font_style(&t.font_weight);
    let display = apply_text_transform(&t.content, t);

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let color = t.color.as_ref()
        .map(super::painter::color_to_skia)
        .unwrap_or(skia_safe::Color4f::new(0.0, 0.0, 0.0, 1.0));
    paint.set_color4f(color, None);

    if t.wrap == Some(true) && rect.w > 0.0 {
        let lines = wrap_shaped_lines(&display, &font, style, fs, rect.w, spacing);
        paint_wrapped(canvas, &lines, &paint, rect, t, spacing);
    } else {
        let shaped = shape_text(&display, &font, style, fs);
        paint_single(canvas, &shaped, &paint, rect, t, spacing);
    }
}

fn paint_single(
    canvas: &Canvas, shaped: &ShapedText,
    paint: &Paint, rect: crate::layout::Rect,
    t: &pastel_lang::ir::node::TextData, spacing: f32,
) {
    let metrics = shaped.primary_metrics();
    let text_h = -metrics.ascent + metrics.descent;
    let ty = rect.y + (rect.h - text_h) / 2.0 - metrics.ascent;

    let total_w = shaped.measure_width_with_spacing(spacing);

    let tx = match t.text_align {
        Some(pastel_lang::ir::style::TextAlign::Center) => rect.x + (rect.w - total_w) / 2.0,
        Some(pastel_lang::ir::style::TextAlign::Right) => rect.x + rect.w - total_w,
        _ => rect.x,
    };

    if spacing.abs() > 0.001 {
        shaped.draw_spaced(canvas, paint, tx, ty, spacing);
    } else {
        shaped.draw(canvas, paint, tx, ty);
    }

    paint_decoration(canvas, t, paint, tx, ty, total_w, &metrics);
}

fn paint_wrapped(
    canvas: &Canvas, lines: &[ShapedText],
    paint: &Paint, rect: crate::layout::Rect,
    t: &pastel_lang::ir::node::TextData, spacing: f32,
) {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let lh = t.line_height.map(|v| v as f32).unwrap_or(fs * 1.3);
    let metrics = lines.first()
        .map(|l| l.primary_metrics())
        .unwrap_or_else(|| skia_safe::Font::default().metrics().1);

    let total_h = lh * lines.len() as f32;
    let start_y = rect.y + (rect.h - total_h) / 2.0 - metrics.ascent;

    for (i, shaped_line) in lines.iter().enumerate() {
        let ty = start_y + lh * i as f32;
        let total_w = shaped_line.measure_width_with_spacing(spacing);

        let tx = match t.text_align {
            Some(pastel_lang::ir::style::TextAlign::Center) => rect.x + (rect.w - total_w) / 2.0,
            Some(pastel_lang::ir::style::TextAlign::Right) => rect.x + rect.w - total_w,
            _ => rect.x,
        };

        if spacing.abs() > 0.001 {
            shaped_line.draw_spaced(canvas, paint, tx, ty, spacing);
        } else {
            shaped_line.draw(canvas, paint, tx, ty);
        }

        paint_decoration(canvas, t, paint, tx, ty, total_w, &metrics);
    }
}

fn paint_decoration(
    canvas: &Canvas, t: &pastel_lang::ir::node::TextData,
    paint: &Paint, tx: f32, ty: f32, text_w: f32,
    metrics: &skia_safe::FontMetrics,
) {
    let dec = match &t.text_decoration {
        Some(d) if *d != TextDecoration::None => d,
        _ => return,
    };
    let mut line_paint = paint.clone();
    line_paint.set_style(skia_safe::PaintStyle::Stroke);
    line_paint.set_stroke_width(1.0);

    let y_pos = match dec {
        TextDecoration::Underline => ty + metrics.descent * 0.5,
        TextDecoration::Strikethrough => ty + metrics.ascent * 0.35,
        TextDecoration::None => unreachable!(),
    };
    canvas.draw_line((tx, y_pos), (tx + text_w, y_pos), &line_paint);
}
