use pastel_lang::ir::node::VisualProps;
use pastel_lang::ir::style::{BlendMode, Fill};

use crate::layout::Rect;

/// Build `transform="rotate(deg, cx, cy)"` attribute string.
pub(super) fn rotation_attr(rotation: Option<f64>, rect: Rect) -> String {
    match rotation {
        Some(deg) if deg.abs() > 0.001 => {
            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            format!(r#" transform="rotate({}, {}, {})""#, deg, cx, cy)
        }
        _ => String::new(),
    }
}

/// Build `style="mix-blend-mode: ..."` attribute string.
pub(super) fn blend_attr(visual: &VisualProps) -> String {
    match &visual.blend {
        Some(mode) => {
            let css = match mode {
                BlendMode::Normal => return String::new(),
                BlendMode::Multiply => "multiply",
                BlendMode::Screen => "screen",
                BlendMode::Overlay => "overlay",
                BlendMode::Darken => "darken",
                BlendMode::Lighten => "lighten",
            };
            format!(r#" style="mix-blend-mode: {}""#, css)
        }
        None => String::new(),
    }
}

/// Build `stroke-dasharray="d,g"` attribute string.
pub(super) fn dash_attr(stroke: Option<&pastel_lang::ir::style::Stroke>) -> String {
    match stroke.and_then(|s| s.dash.as_ref()) {
        Some(dash) => format!(r#" stroke-dasharray="{},{}""#, dash[0], dash[1]),
        None => String::new(),
    }
}

/// Write blur filter defs and return the filter attribute string.
pub(super) fn blur_filter(visual: &VisualProps, id: &str, defs: &mut String) -> String {
    if let Some(blur) = visual.blur {
        if blur > 0.0 {
            let filter_id = format!("blur-{}", id);
            let sigma = blur / 2.0;
            defs.push_str(&format!(
                r#"    <filter id="{fid}"><feGaussianBlur in="SourceGraphic" stdDeviation="{sigma}" /></filter>"#,
                fid = filter_id,
            ));
            defs.push('\n');
            return format!(r#" filter="url(#{})""#, filter_id);
        }
    }
    if let Some(bg_blur) = visual.background_blur {
        if bg_blur > 0.0 {
            let filter_id = format!("bgblur-{}", id);
            let sigma = bg_blur / 2.0;
            defs.push_str(&format!(
                r#"    <filter id="{fid}"><feGaussianBlur in="BackgroundImage" stdDeviation="{sigma}" /></filter>"#,
                fid = filter_id,
            ));
            defs.push('\n');
            return format!(r#" filter="url(#{})""#, filter_id);
        }
    }
    String::new()
}

/// Resolve fill to SVG fill attribute value, writing gradient defs as needed.
pub(super) fn fill_str(fill: &Fill, id: &str, defs: &mut String) -> String {
    match fill {
        Fill::Solid { color } => color.to_hex(),
        Fill::Transparent => "none".into(),
        Fill::LinearGradient { angle, stops } => {
            let grad_id = format!("grad-{}", id);
            write_gradient_def(&grad_id, *angle, stops, defs);
            format!("url(#{})", grad_id)
        }
        Fill::RadialGradient { cx, cy, stops } => {
            let grad_id = format!("rgrad-{}", id);
            write_radial_gradient_def(&grad_id, *cx, *cy, stops, defs);
            format!("url(#{})", grad_id)
        }
    }
}

fn write_gradient_def(
    id: &str,
    angle: f64,
    stops: &[pastel_lang::ir::style::GradientStop],
    defs: &mut String,
) {
    use std::f64::consts::PI;
    let rad = angle * PI / 180.0;
    let (x1, y1) = (50.0 - 50.0 * rad.sin(), 50.0 + 50.0 * rad.cos());
    let (x2, y2) = (50.0 + 50.0 * rad.sin(), 50.0 - 50.0 * rad.cos());
    defs.push_str(&format!(
        r#"    <linearGradient id="{id}" x1="{x1:.1}%" y1="{y1:.1}%" x2="{x2:.1}%" y2="{y2:.1}%">"#,
    ));
    defs.push('\n');
    for stop in stops {
        defs.push_str(&format!(
            r#"      <stop offset="{pos}%" stop-color="{color}" />"#,
            pos = stop.position,
            color = stop.color.to_hex(),
        ));
        defs.push('\n');
    }
    defs.push_str("    </linearGradient>\n");
}

fn write_radial_gradient_def(
    id: &str,
    cx: f64,
    cy: f64,
    stops: &[pastel_lang::ir::style::GradientStop],
    defs: &mut String,
) {
    defs.push_str(&format!(
        r#"    <radialGradient id="{id}" cx="{cx:.1}%" cy="{cy:.1}%" r="50%">"#,
    ));
    defs.push('\n');
    for stop in stops {
        defs.push_str(&format!(
            r#"      <stop offset="{pos}%" stop-color="{color}" />"#,
            pos = stop.position,
            color = stop.color.to_hex(),
        ));
        defs.push('\n');
    }
    defs.push_str("    </radialGradient>\n");
}

pub(super) fn write_shadow_def(
    defs: &mut String,
    id: &str,
    shadow: &pastel_lang::ir::style::Shadow,
) {
    defs.push_str(&format!(
        r#"    <filter id="shadow-{id}"><feDropShadow dx="{dx}" dy="{dy}" stdDeviation="{std}" flood-color="{color}" flood-opacity="{opacity}" /></filter>"#,
        dx = shadow.x, dy = shadow.y, std = shadow.blur / 2.0,
        color = shadow.color.to_hex(), opacity = shadow.color.a as f32 / 255.0,
    ));
    defs.push('\n');
}

pub(super) fn shadow_filter_attr(
    shadow: Option<&pastel_lang::ir::style::Shadow>,
    id: &str,
) -> String {
    if shadow.is_some() {
        format!(r#" filter="url(#shadow-{})""#, id)
    } else {
        String::new()
    }
}
