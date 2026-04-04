use std::collections::HashMap;

use pastel_lang::ir::node::{IrNode, IrNodeData, FrameData, TextData};
use pastel_lang::ir::style::{Dimension, LayoutMode, PositionMode};
use pastel_lang::ir::IrDocument;
use skia_safe::{Canvas, Font, FontMgr, FontStyle};

use crate::layout_place::place_children;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct LayoutTree {
    pub rects: HashMap<String, Rect>,
}

impl LayoutTree {
    pub fn get(&self, id: &str) -> Option<&Rect> {
        self.rects.get(id)
    }

    pub fn compute(doc: &IrDocument, canvas: &Canvas) -> Self {
        let mut tree = LayoutTree { rects: HashMap::new() };
        let cw = doc.canvas.width as f32;
        let ch = doc.canvas.height as f32;

        let mut y = 0.0;
        for node in &doc.nodes {
            let size = measure(node, cw, ch - y, canvas);
            let w = resolve_main(size.w, size.w_fill, cw);
            let h = resolve_main(size.h, size.h_fill, ch - y);
            tree.rects.insert(node.id.clone(), Rect { x: 0.0, y, w, h });
            place_children(node, 0.0, y, w, h, &mut tree, canvas);
            y += h;
        }
        tree
    }
}

// -- Measurement (pub(crate) for layout_place) --

pub(crate) struct Size { pub w: f32, pub h: f32, pub w_fill: bool, pub h_fill: bool }

pub(crate) fn measure(node: &IrNode, aw: f32, ah: f32, c: &Canvas) -> Size {
    match &node.data {
        IrNodeData::Frame(f) => measure_frame(node, f, aw, ah, c),
        IrNodeData::Text(t) => measure_text(t, aw),
        IrNodeData::Image(img) => {
            let (w, wf) = dim(img.width.as_ref(), aw);
            let (h, hf) = dim(img.height.as_ref(), ah);
            Size { w, h, w_fill: wf, h_fill: hf }
        }
        IrNodeData::Shape(s) => {
            let (w, wf) = dim(s.width.as_ref(), aw);
            let (h, hf) = dim(s.height.as_ref(), ah);
            Size { w, h, w_fill: wf, h_fill: hf }
        }
    }
}

fn dim(d: Option<&Dimension>, parent: f32) -> (f32, bool) {
    match d {
        Some(Dimension::Fixed(n)) => (*n as f32, false),
        Some(Dimension::Fill) => (parent, true),
        _ => (0.0, false),
    }
}

pub(crate) fn resolve_main(val: f32, is_fill: bool, available: f32) -> f32 {
    if is_fill { available } else { val }
}

pub(crate) fn pad(f: &FrameData) -> [f32; 4] {
    f.padding.as_ref().map(|p| p.0.map(|v| v as f32)).unwrap_or([0.0; 4])
}

pub(crate) fn is_absolute(node: &IrNode) -> bool {
    if let IrNodeData::Frame(f) = &node.data {
        matches!(f.position.as_ref().map(|p| &p.mode), Some(PositionMode::Absolute))
    } else { false }
}

/// Apply text transform to content string.
pub fn apply_text_transform(content: &str, t: &TextData) -> String {
    match &t.text_transform {
        Some(pastel_lang::ir::style::TextTransform::Uppercase) => content.to_uppercase(),
        Some(pastel_lang::ir::style::TextTransform::Lowercase) => content.to_lowercase(),
        _ => content.to_string(),
    }
}

fn measure_text(t: &TextData, available_w: f32) -> Size {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let spacing = t.letter_spacing.unwrap_or(0.0) as f32;
    let font = make_font(t.font_family.as_deref(), &t.font_weight, fs);
    let display = apply_text_transform(&t.content, t);

    let text_width = t.width.as_ref().and_then(|d| match d {
        Dimension::Fixed(n) => Some(*n as f32),
        Dimension::Fill => Some(available_w),
        _ => None,
    });

    let char_count = display.chars().count().max(1) as f32;
    let extra_spacing = spacing * (char_count - 1.0).max(0.0);

    if t.wrap == Some(true) && text_width.is_some() {
        let max_w = text_width.unwrap();
        let lines = word_wrap_lines(&display, &font, max_w, spacing);
        let lh = t.line_height.map(|v| v as f32).unwrap_or(fs * 1.3);
        let h = lh * lines.len() as f32;
        let explicit_h = t.height.as_ref().and_then(|d| match d {
            Dimension::Fixed(n) => Some(*n as f32), _ => None,
        });
        Size { w: max_w, h: explicit_h.unwrap_or(h), w_fill: false, h_fill: false }
    } else {
        let (tw, _) = font.measure_str(&display, None);
        let w = text_width.unwrap_or(tw.ceil() + 2.0 + extra_spacing);
        let explicit_h = t.height.as_ref().and_then(|d| match d {
            Dimension::Fixed(n) => Some(*n as f32), _ => None,
        });
        Size { w, h: explicit_h.unwrap_or(fs * 1.3), w_fill: false, h_fill: false }
    }
}

/// Break text into lines that fit within max_width using word boundaries.
pub fn word_wrap_lines(text: &str, font: &Font, max_w: f32, spacing: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        if words.is_empty() { lines.push(String::new()); continue; }
        let mut line = String::new();
        for word in &words {
            let candidate = if line.is_empty() { word.to_string() }
            else { format!("{} {}", line, word) };
            let cc = candidate.chars().count().max(1) as f32;
            let extra = spacing * (cc - 1.0).max(0.0);
            let (w, _) = font.measure_str(&candidate, None);
            if w + extra > max_w && !line.is_empty() {
                lines.push(line);
                line = word.to_string();
            } else { line = candidate; }
        }
        if !line.is_empty() { lines.push(line); }
    }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}

fn measure_frame(node: &IrNode, f: &FrameData, aw: f32, ah: f32, c: &Canvas) -> Size {
    let (mut w, wf) = dim(f.width.as_ref(), aw);
    let (mut h, hf) = dim(f.height.as_ref(), ah);
    let p = pad(f);

    let inner_w = if wf || w > 0.0 { (if wf { aw } else { w }) - p[1] - p[3] } else { aw };
    let inner_h = if hf || h > 0.0 { (if hf { ah } else { h }) - p[0] - p[2] } else { ah };

    let flow: Vec<&IrNode> = node.children.iter()
        .filter(|ch| !is_absolute(ch)).collect();
    if flow.is_empty() {
        return Size { w, h, w_fill: wf, h_fill: hf };
    }

    let layout = f.layout.as_ref();
    let is_h = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Horizontal));
    let gap = layout.and_then(|l| l.gap).unwrap_or(0.0) as f32;

    let sizes: Vec<Size> = flow.iter()
        .map(|ch| measure(ch, inner_w, inner_h, c)).collect();

    let tg = gap * sizes.len().saturating_sub(1) as f32;
    let (cw, ch_) = if is_h {
        (sizes.iter().map(|s| s.w).sum::<f32>() + tg,
         sizes.iter().map(|s| s.h).fold(0.0f32, f32::max))
    } else {
        (sizes.iter().map(|s| s.w).fold(0.0f32, f32::max),
         sizes.iter().map(|s| s.h).sum::<f32>() + tg)
    };

    if w == 0.0 && !wf { w = cw + p[1] + p[3]; }
    if h == 0.0 && !hf { h = ch_ + p[0] + p[2]; }

    Size { w, h, w_fill: wf, h_fill: hf }
}

// -- Font --

pub fn make_font(
    family: Option<&str>, weight: &Option<pastel_lang::ir::style::FontWeight>, size: f32,
) -> Font {
    use pastel_lang::ir::style::FontWeight as FW;
    let style = match weight {
        Some(w) => {
            let wv = match w {
                FW::Thin => skia_safe::font_style::Weight::THIN,
                FW::Light => skia_safe::font_style::Weight::LIGHT,
                FW::Normal => skia_safe::font_style::Weight::NORMAL,
                FW::Medium => skia_safe::font_style::Weight::MEDIUM,
                FW::Semibold => skia_safe::font_style::Weight::SEMI_BOLD,
                FW::Bold => skia_safe::font_style::Weight::BOLD,
                FW::Extrabold => skia_safe::font_style::Weight::EXTRA_BOLD,
                FW::Black => skia_safe::font_style::Weight::BLACK,
            };
            FontStyle::new(wv, skia_safe::font_style::Width::NORMAL, skia_safe::font_style::Slant::Upright)
        }
        None => FontStyle::normal(),
    };
    let fm = FontMgr::default();
    fm.match_family_style(family.unwrap_or("Helvetica"), style)
        .or_else(|| fm.match_family_style("Arial", style))
        .or_else(|| fm.match_family_style("sans-serif", style))
        .map(|tf| Font::from_typeface(tf, size))
        .unwrap_or_else(|| {
            let tf = fm.legacy_make_typeface(None, FontStyle::normal()).expect("no fonts");
            Font::from_typeface(tf, size)
        })
}
