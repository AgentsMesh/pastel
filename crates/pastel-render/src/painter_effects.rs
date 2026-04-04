use pastel_lang::ir::node::VisualProps;
use pastel_lang::ir::style::{BlendMode, Stroke};
use skia_safe::{Canvas, Paint, Rect, RRect, PathEffect};

use crate::layout;
use crate::painter::{color_to_skia, to_sk_rect, corner_radii};

/// Apply rotation transform around the center of the rect. Returns true if save was called.
pub(crate) fn apply_rotation(canvas: &Canvas, rotation: Option<f64>, rect: layout::Rect) -> bool {
    if let Some(deg) = rotation {
        if deg.abs() > 0.001 {
            canvas.save();
            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            canvas.rotate(deg as f32, Some(skia_safe::Point::new(cx, cy)));
            return true;
        }
    }
    false
}

/// Apply blend mode by saving a layer with the appropriate mode. Returns true if layer was saved.
pub(crate) fn apply_blend_mode(canvas: &Canvas, visual: &VisualProps) -> bool {
    if let Some(blend) = &visual.blend {
        let sk_blend = match blend {
            BlendMode::Normal => return false,
            BlendMode::Multiply => skia_safe::BlendMode::Multiply,
            BlendMode::Screen => skia_safe::BlendMode::Screen,
            BlendMode::Overlay => skia_safe::BlendMode::Overlay,
            BlendMode::Darken => skia_safe::BlendMode::Darken,
            BlendMode::Lighten => skia_safe::BlendMode::Lighten,
        };
        let mut paint = Paint::default();
        paint.set_blend_mode(sk_blend);
        canvas.save_layer_alpha_f(None, 1.0);
        // We need to set blend mode on the layer — use save_layer with paint
        canvas.restore(); // undo the alpha layer
        canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&paint));
        return true;
    }
    false
}

/// Apply gaussian blur image filter to paint if blur is set.
pub(crate) fn apply_blur_filter(paint: &mut Paint, visual: &VisualProps) {
    if let Some(blur) = visual.blur {
        if blur > 0.0 {
            let sigma = blur as f32 / 2.0;
            if let Some(filter) = skia_safe::image_filters::blur(
                (sigma, sigma), skia_safe::TileMode::Clamp, None, None,
            ) {
                paint.set_image_filter(filter);
            }
        }
    }
}

/// Apply dash path effect to a stroke paint.
pub(crate) fn apply_dash_effect(paint: &mut Paint, stroke: &Stroke) {
    if let Some(dash) = &stroke.dash {
        let intervals = [dash[0] as f32, dash[1] as f32];
        if let Some(effect) = PathEffect::dash(&intervals, 0.0) {
            paint.set_path_effect(effect);
        }
    }
}

/// Render inner shadow by clipping to the shape and drawing an inverted shadow.
pub(crate) fn paint_inner_shadow(
    canvas: &Canvas, shadow: &pastel_lang::ir::style::Shadow,
    rect: layout::Rect, cr: Option<[f32; 4]>,
) {
    let sk_rect = to_sk_rect(rect);

    canvas.save();

    // Clip to the shape bounds
    if let Some(radii) = cr {
        let rrect = RRect::new_rect_radii(sk_rect, &corner_radii(radii, rect.w, rect.h));
        canvas.clip_rrect(rrect, skia_safe::ClipOp::Intersect, true);
    } else {
        canvas.clip_rect(sk_rect, skia_safe::ClipOp::Intersect, Some(true));
    }

    // Draw a large rect offset outside the shape to create inner shadow effect
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(color_to_skia(&shadow.color), None);
    let blur = shadow.blur as f32;
    if blur > 0.0 {
        paint.set_mask_filter(skia_safe::MaskFilter::blur(
            skia_safe::BlurStyle::Normal, blur / 2.0, false,
        ));
    }
    paint.set_style(skia_safe::PaintStyle::Stroke);
    // Use a thick stroke around the rect offset by shadow x/y
    let spread = blur * 2.0 + rect.w.max(rect.h);
    paint.set_stroke_width(spread);

    let offset_rect = Rect::from_xywh(
        rect.x + shadow.x as f32 - spread / 2.0,
        rect.y + shadow.y as f32 - spread / 2.0,
        rect.w + spread,
        rect.h + spread,
    );
    canvas.draw_rect(offset_rect, &paint);

    canvas.restore();
}
