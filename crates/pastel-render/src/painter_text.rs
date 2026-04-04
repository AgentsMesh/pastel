use pastel_lang::ir::style::TextDecoration;
use skia_safe::{Canvas, Paint, TextBlob};

use crate::layout::{apply_text_transform, make_font, word_wrap_lines};

/// Paint a text node (single-line or wrapped).
pub(crate) fn paint_text(
    canvas: &Canvas, t: &pastel_lang::ir::node::TextData, rect: crate::layout::Rect,
) {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let spacing = t.letter_spacing.unwrap_or(0.0) as f32;
    let font = make_font(t.font_family.as_deref(), &t.font_weight, fs);
    let display = apply_text_transform(&t.content, t);

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let color = t.color.as_ref()
        .map(super::painter::color_to_skia)
        .unwrap_or(skia_safe::Color4f::new(0.0, 0.0, 0.0, 1.0));
    paint.set_color4f(color, None);

    if t.wrap == Some(true) && rect.w > 0.0 {
        paint_wrapped(canvas, &display, &font, &paint, rect, t, spacing);
    } else {
        paint_single(canvas, &display, &font, &paint, rect, t, spacing);
    }
}

fn paint_single(
    canvas: &Canvas, display: &str, font: &skia_safe::Font,
    paint: &Paint, rect: crate::layout::Rect,
    t: &pastel_lang::ir::node::TextData, spacing: f32,
) {
    let metrics = font.metrics();
    let text_h = -metrics.1.ascent + metrics.1.descent;
    let ty = rect.y + (rect.h - text_h) / 2.0 - metrics.1.ascent;

    let (text_w, _) = font.measure_str(display, None);
    let cc = display.chars().count().max(1) as f32;
    let extra = spacing * (cc - 1.0).max(0.0);
    let total_w = text_w + extra;

    let tx = match t.text_align {
        Some(pastel_lang::ir::style::TextAlign::Center) => rect.x + (rect.w - total_w) / 2.0,
        Some(pastel_lang::ir::style::TextAlign::Right) => rect.x + rect.w - total_w,
        _ => rect.x,
    };

    if spacing.abs() > 0.001 {
        draw_spaced(canvas, display, font, paint, tx, ty, spacing);
    } else if let Some(blob) = TextBlob::from_str(display, font) {
        canvas.draw_text_blob(&blob, (tx, ty), paint);
    }

    paint_decoration(canvas, t, paint, tx, ty, total_w, &metrics.1);
}

fn paint_wrapped(
    canvas: &Canvas, display: &str, font: &skia_safe::Font,
    paint: &Paint, rect: crate::layout::Rect,
    t: &pastel_lang::ir::node::TextData, spacing: f32,
) {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let lh = t.line_height.map(|v| v as f32).unwrap_or(fs * 1.3);
    let lines = word_wrap_lines(display, font, rect.w, spacing);
    let metrics = font.metrics();

    let total_h = lh * lines.len() as f32;
    let start_y = rect.y + (rect.h - total_h) / 2.0 - metrics.1.ascent;

    for (i, line) in lines.iter().enumerate() {
        let ty = start_y + lh * i as f32;
        let (line_w, _) = font.measure_str(line, None);
        let cc = line.chars().count().max(1) as f32;
        let extra = spacing * (cc - 1.0).max(0.0);
        let total_w = line_w + extra;

        let tx = match t.text_align {
            Some(pastel_lang::ir::style::TextAlign::Center) => rect.x + (rect.w - total_w) / 2.0,
            Some(pastel_lang::ir::style::TextAlign::Right) => rect.x + rect.w - total_w,
            _ => rect.x,
        };

        if spacing.abs() > 0.001 {
            draw_spaced(canvas, line, font, paint, tx, ty, spacing);
        } else if let Some(blob) = TextBlob::from_str(line, font) {
            canvas.draw_text_blob(&blob, (tx, ty), paint);
        }

        paint_decoration(canvas, t, paint, tx, ty, total_w, &metrics.1);
    }
}

fn draw_spaced(
    canvas: &Canvas, text: &str, font: &skia_safe::Font,
    paint: &Paint, mut x: f32, y: f32, spacing: f32,
) {
    for ch in text.chars() {
        let s = ch.to_string();
        if let Some(blob) = TextBlob::from_str(&s, font) {
            canvas.draw_text_blob(&blob, (x, y), paint);
        }
        let (cw, _) = font.measure_str(&s, None);
        x += cw + spacing;
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
