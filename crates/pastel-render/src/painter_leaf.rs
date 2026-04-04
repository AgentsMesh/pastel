use skia_safe::{Canvas, Paint, Color4f, Rect, RRect, TextBlob};
use skia_safe::gradient_shader::GradientShaderColors;

use crate::layout::{self, make_font};
use crate::painter::{color_to_skia, to_sk_rect, corner_radii};

/// Paint a drop shadow behind a rect.
pub(crate) fn paint_shadow(
    canvas: &Canvas, shadow: &pastel_lang::ir::style::Shadow,
    rect: layout::Rect, cr: Option<[f32; 4]>,
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

/// Paint an image placeholder.
pub(crate) fn paint_image(
    canvas: &Canvas, img: &pastel_lang::ir::node::ImageData, rect: layout::Rect,
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

/// Create a radial gradient shader.
pub(crate) fn make_radial_gradient_shader(
    cx_pct: f64, cy_pct: f64,
    stops: &[pastel_lang::ir::style::GradientStop], rect: Rect,
) -> Option<skia_safe::Shader> {
    use skia_safe::{Point, gradient_shader, TileMode};

    let cx = rect.x() + rect.width() * (cx_pct as f32 / 100.0);
    let cy = rect.y() + rect.height() * (cy_pct as f32 / 100.0);
    let radius = rect.width().max(rect.height()) / 2.0;

    let colors: Vec<Color4f> = stops.iter().map(|s| color_to_skia(&s.color)).collect();
    let pos: Vec<f32> = stops.iter().map(|s| (s.position / 100.0) as f32).collect();

    gradient_shader::radial(
        Point::new(cx, cy), radius,
        GradientShaderColors::ColorsInSpace(&colors, None),
        pos.as_slice(), TileMode::Clamp, None, None,
    )
}

/// Create a linear gradient shader.
pub(crate) fn make_gradient_shader(
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
