use std::collections::HashMap;

use pastel_lang::ir::node::{IrNode, IrNodeData, FrameData, TextData};
use pastel_lang::ir::style::{Dimension, LayoutMode, Align, Justify};
use pastel_lang::ir::IrDocument;
use skia_safe::{Canvas, Font, FontMgr, FontStyle};

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

        // Document-level: stack top-level nodes vertically
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

// -- Measurement --

struct Size { w: f32, h: f32, w_fill: bool, h_fill: bool }

fn measure(node: &IrNode, aw: f32, ah: f32, c: &Canvas) -> Size {
    match &node.data {
        IrNodeData::Frame(f) => measure_frame(node, f, aw, ah, c),
        IrNodeData::Text(t) => measure_text(t),
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

fn resolve_main(val: f32, is_fill: bool, available: f32) -> f32 {
    if is_fill { available } else { val }
}

fn pad(f: &FrameData) -> [f32; 4] {
    f.padding.as_ref().map(|p| p.0.map(|v| v as f32)).unwrap_or([0.0; 4])
}

fn measure_text(t: &TextData) -> Size {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let font = make_font(t.font_family.as_deref(), &t.font_weight, fs);
    let (tw, _) = font.measure_str(&t.content, None);
    Size { w: tw.ceil() + 2.0, h: fs * 1.3, w_fill: false, h_fill: false }
}

fn measure_frame(node: &IrNode, f: &FrameData, aw: f32, ah: f32, c: &Canvas) -> Size {
    let (mut w, wf) = dim(f.width.as_ref(), aw);
    let (mut h, hf) = dim(f.height.as_ref(), ah);
    let p = pad(f);

    // Available inner space for children measurement
    let inner_w = if wf || w > 0.0 { (if wf { aw } else { w }) - p[1] - p[3] } else { aw };
    let inner_h = if hf || h > 0.0 { (if hf { ah } else { h }) - p[0] - p[2] } else { ah };

    if node.children.is_empty() {
        return Size { w, h, w_fill: wf, h_fill: hf };
    }

    // Measure children with correct available space
    let layout = f.layout.as_ref();
    let is_h = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Horizontal));
    let gap = layout.and_then(|l| l.gap).unwrap_or(0.0) as f32;

    let sizes: Vec<Size> = node.children.iter()
        .map(|ch| measure(ch, inner_w, inner_h, c))
        .collect();

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

// -- Placement --

fn place_children(
    node: &IrNode, px: f32, py: f32, pw: f32, ph: f32,
    tree: &mut LayoutTree, c: &Canvas,
) {
    if node.children.is_empty() { return; }
    let (p, layout) = match &node.data {
        IrNodeData::Frame(f) => (pad(f), f.layout.as_ref()),
        _ => return,
    };

    let ix = px + p[3];
    let iy = py + p[0];
    let iw = (pw - p[1] - p[3]).max(0.0);
    let ih = (ph - p[0] - p[2]).max(0.0);

    let is_h = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Horizontal));
    let gap = layout.and_then(|l| l.gap).unwrap_or(0.0) as f32;
    let align = layout.and_then(|l| l.align.as_ref());
    let justify = layout.and_then(|l| l.justify.as_ref());

    // Measure children in context of INNER available space
    let mut sizes: Vec<Size> = node.children.iter()
        .map(|ch| measure(ch, iw, ih, c))
        .collect();

    // Resolve fill: cross-axis
    for s in &mut sizes {
        if is_h && s.h_fill { s.h = ih; }
        if !is_h && s.w_fill { s.w = iw; }
    }

    // Resolve fill: main-axis (distribute remaining space)
    let fixed: f32 = sizes.iter()
        .map(|s| if (is_h && !s.w_fill) || (!is_h && !s.h_fill) {
            if is_h { s.w } else { s.h }
        } else { 0.0 })
        .sum();
    let tg = gap * sizes.len().saturating_sub(1) as f32;
    let free = (if is_h { iw } else { ih }) - fixed - tg;
    let fill_n = sizes.iter()
        .filter(|s| if is_h { s.w_fill } else { s.h_fill }).count();
    if fill_n > 0 && free > 0.0 {
        let each = free / fill_n as f32;
        for s in &mut sizes {
            if is_h && s.w_fill { s.w = each; }
            if !is_h && s.h_fill { s.h = each; }
        }
    }

    // Total after resolving fills
    let total: f32 = sizes.iter()
        .map(|s| if is_h { s.w } else { s.h }).sum::<f32>() + tg;
    let fs = (if is_h { iw } else { ih }) - total;

    let (mut cx, mut cy) = (ix, iy);
    let mut sb = 0.0f32;

    match justify {
        Some(Justify::Center) => { if is_h { cx += fs / 2.0 } else { cy += fs / 2.0 } }
        Some(Justify::End) => { if is_h { cx += fs } else { cy += fs } }
        Some(Justify::SpaceBetween) if node.children.len() > 1 => {
            sb = fs / (node.children.len() - 1) as f32;
        }
        Some(Justify::SpaceAround) if !node.children.is_empty() => {
            let ar = fs / (node.children.len() * 2) as f32;
            if is_h { cx += ar } else { cy += ar }
            sb = ar;
        }
        _ => {}
    }

    for (i, child) in node.children.iter().enumerate() {
        let s = &sizes[i];
        let (mut nx, mut ny) = (cx, cy);

        if is_h {
            match align {
                Some(Align::Center) => ny = iy + (ih - s.h) / 2.0,
                Some(Align::End) => ny = iy + ih - s.h,
                _ => {}
            }
        } else {
            match align {
                Some(Align::Center) => nx = ix + (iw - s.w) / 2.0,
                Some(Align::End) => nx = ix + iw - s.w,
                _ => {}
            }
        }

        tree.rects.insert(child.id.clone(), Rect { x: nx, y: ny, w: s.w, h: s.h });
        place_children(child, nx, ny, s.w, s.h, tree, c);

        let step = (if is_h { s.w } else { s.h }) + gap + sb;
        if is_h { cx += step } else { cy += step }
    }
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
