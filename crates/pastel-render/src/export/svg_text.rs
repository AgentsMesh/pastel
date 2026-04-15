use base64::Engine;
use pastel_lang::ir::node::IrNode;
use pastel_lang::ir::style::TextDecoration;
use pastel_lang::ir::IrAsset;

use crate::layout::{Rect, apply_text_transform, make_font, word_wrap_lines};

/// Render a text node to SVG.
pub(super) fn render_text(
    t: &pastel_lang::ir::node::TextData, rect: Rect,
    _node: &IrNode, out: &mut String, _defs: &mut String, indent: &str,
) {
    let fs = t.font_size.unwrap_or(14.0);
    let fw = t.font_weight.as_ref()
        .map(|w| format!("{:?}", w).to_lowercase())
        .unwrap_or_else(|| "normal".into());
    let ff = t.font_family.as_deref().unwrap_or("sans-serif");
    let color = t.color.as_ref().map(|c| c.to_hex()).unwrap_or_else(|| "#000000".into());
    let display = apply_text_transform(&t.content, t);
    let spacing = t.letter_spacing.unwrap_or(0.0);

    let font = make_font(t.font_family.as_deref(), &t.font_weight, fs as f32);
    let metrics = font.metrics();
    let text_h = -metrics.1.ascent + metrics.1.descent;

    let spacing_attr = if spacing.abs() > 0.001 {
        format!(r#" letter-spacing="{}""#, spacing)
    } else { String::new() };

    let decoration_attr = match &t.text_decoration {
        Some(TextDecoration::Underline) => r#" text-decoration="underline""#.to_string(),
        Some(TextDecoration::Strikethrough) => r#" text-decoration="line-through""#.to_string(),
        _ => String::new(),
    };

    if t.wrap == Some(true) && rect.w > 0.0 {
        let lh = t.line_height.map(|v| v as f32).unwrap_or(fs as f32 * 1.3);
        let lines = word_wrap_lines(&display, &font, rect.w, spacing as f32);
        let total_h = lh * lines.len() as f32;
        let start_y = rect.y + (rect.h - total_h) / 2.0 - metrics.1.ascent;

        for (i, line) in lines.iter().enumerate() {
            let ty = start_y + lh * i as f32;
            let (tx, anchor) = text_anchor(t, rect);
            out.push_str(&format!(
                r#"{indent}<text x="{tx}" y="{ty}" font-size="{fs}" font-weight="{fw}" font-family="{ff}" fill="{color}" text-anchor="{anchor}"{spacing}{decoration}>{content}</text>"#,
                content = escape_xml(line), spacing = spacing_attr,
                decoration = decoration_attr,
            ));
            out.push('\n');
        }
    } else {
        let ty = rect.y + (rect.h - text_h) / 2.0 - metrics.1.ascent;
        let (tx, anchor) = text_anchor(t, rect);

        out.push_str(&format!(
            r#"{indent}<text x="{tx}" y="{ty}" font-size="{fs}" font-weight="{fw}" font-family="{ff}" fill="{color}" text-anchor="{anchor}"{spacing}{decoration}>{content}</text>"#,
            content = escape_xml(&display), spacing = spacing_attr,
            decoration = decoration_attr,
        ));
        out.push('\n');
    }
}

fn text_anchor(
    t: &pastel_lang::ir::node::TextData, rect: Rect,
) -> (f32, &'static str) {
    match t.text_align {
        Some(pastel_lang::ir::style::TextAlign::Center) => (rect.x + rect.w / 2.0, "middle"),
        Some(pastel_lang::ir::style::TextAlign::Right) => (rect.x + rect.w, "end"),
        _ => (rect.x, "start"),
    }
}

pub(super) fn render_image(
    img: &pastel_lang::ir::node::ImageData, rect: Rect,
    out: &mut String, indent: &str, corner_radius_fn: fn(Option<&[f64; 4]>, Rect) -> String,
    assets: &[IrAsset],
) {
    // Try to embed real image as base64 data URI
    if let Some(asset) = assets.iter().find(|a| a.id == img.asset) {
        if let Some(ref path) = asset.resolved_path {
            if let Ok(bytes) = std::fs::read(path) {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let mime = match ext {
                    "png" => "image/png",
                    "svg" => "image/svg+xml",
                    "webp" => "image/webp",
                    "gif" => "image/gif",
                    _ => "image/jpeg",
                };
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let radius_attr = corner_radius_fn(img.corner_radius.as_ref().map(|r| &r.0), rect);
                let preserve = match img.fit {
                    Some(pastel_lang::ir::extra::ImageFit::Contain) => "xMidYMid meet",
                    Some(pastel_lang::ir::extra::ImageFit::None) => "none",
                    _ => "xMidYMid slice",
                };
                // Clip with rounded rect if needed
                let has_radius = img.corner_radius.as_ref().map(|r| r.0.iter().any(|v| *v > 0.0)).unwrap_or(false);
                if has_radius {
                    let clip_id = format!("clip-{}-{}", rect.x as i32, rect.y as i32);
                    out.push_str(&format!(
                        "{indent}<clipPath id=\"{clip_id}\"><rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"{radius} /></clipPath>\n",
                        x = rect.x, y = rect.y, w = rect.w, h = rect.h, radius = radius_attr,
                    ));
                    out.push_str(&format!(
                        "{indent}<image x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" href=\"data:{mime};base64,{b64}\" preserveAspectRatio=\"{preserve}\" clip-path=\"url(#{clip_id})\" />\n",
                        x = rect.x, y = rect.y, w = rect.w, h = rect.h,
                    ));
                } else {
                    out.push_str(&format!(
                        "{indent}<image x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" href=\"data:{mime};base64,{b64}\" preserveAspectRatio=\"{preserve}\" />\n",
                        x = rect.x, y = rect.y, w = rect.w, h = rect.h,
                    ));
                }
                return;
            }
        }
    }

    // Fallback: placeholder
    let radius_attr = corner_radius_fn(img.corner_radius.as_ref().map(|r| &r.0), rect);
    out.push_str(&format!(
        "{indent}<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"#E8E8E8\" stroke=\"#D0D0D0\"{radius} />",
        x = rect.x, y = rect.y, w = rect.w, h = rect.h, radius = radius_attr,
    ));
    out.push('\n');
    let label = img.name.as_deref().unwrap_or("image");
    out.push_str(&format!(
        "{indent}<text x=\"{tx}\" y=\"{ty}\" font-size=\"12\" fill=\"#AAAAAA\" text-anchor=\"middle\" dominant-baseline=\"central\">{label}</text>",
        tx = rect.x + rect.w / 2.0, ty = rect.y + rect.h / 2.0,
    ));
    out.push('\n');
}

pub(super) fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
