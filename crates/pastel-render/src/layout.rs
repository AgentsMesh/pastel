use std::collections::HashMap;

use pastel_lang::ir::node::{IrNode, IrNodeData, FrameData, TextData};
use pastel_lang::ir::style::{Dimension, LayoutMode, Align, Justify};
use pastel_lang::ir::IrDocument;
use skia_safe::{Canvas, Font, FontMgr, FontStyle};

/// Computed absolute position and size for each node.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Map of node ID → computed layout rect.
pub struct LayoutTree {
    pub rects: HashMap<String, Rect>,
}

impl LayoutTree {
    pub fn get(&self, id: &str) -> Option<&Rect> {
        self.rects.get(id)
    }

    /// Compute layout for the entire document.
    pub fn compute(doc: &IrDocument, canvas: &Canvas) -> Self {
        let mut tree = LayoutTree {
            rects: HashMap::new(),
        };
        let cw = doc.canvas.width as f32;
        let ch = doc.canvas.height as f32;

        // Document-level: stack top-level nodes vertically
        let mut y = 0.0;
        for node in &doc.nodes {
            let size = measure_node(node, cw, ch - y, canvas);
            let w = if size.w_fill { cw } else { size.w.max(cw) };
            let h = size.h;
            tree.rects.insert(node.id.clone(), Rect { x: 0.0, y, w, h });
            layout_children(node, 0.0, y, w, h, &mut tree, canvas);
            y += h;
        }
        tree
    }
}

struct MeasuredSize {
    w: f32,
    h: f32,
    w_fill: bool,
    h_fill: bool,
}

fn dim_val(dim: &Option<Dimension>, parent: f32) -> (f32, bool) {
    match dim {
        Some(Dimension::Fixed(n)) => (*n as f32, false),
        Some(Dimension::Fill) => (parent, true),
        Some(Dimension::Hug) | None => (0.0, false),
    }
}

fn measure_node(node: &IrNode, pw: f32, ph: f32, canvas: &Canvas) -> MeasuredSize {
    match &node.data {
        IrNodeData::Frame(f) => measure_frame(node, f, pw, ph, canvas),
        IrNodeData::Text(t) => measure_text(t, canvas),
        IrNodeData::Image(img) => {
            let (w, wf) = dim_val(&img.width, pw);
            let (h, hf) = dim_val(&img.height, ph);
            MeasuredSize { w, h, w_fill: wf, h_fill: hf }
        }
        IrNodeData::Shape(_) => MeasuredSize { w: 0.0, h: 0.0, w_fill: false, h_fill: false },
    }
}

fn measure_text(t: &TextData, _canvas: &Canvas) -> MeasuredSize {
    let fs = t.font_size.unwrap_or(14.0) as f32;
    let font = make_font(t.font_family.as_deref(), &t.font_weight, fs);
    let (text_w, _) = font.measure_str(&t.content, None);
    MeasuredSize {
        w: text_w.ceil() + 2.0,
        h: fs * 1.3,
        w_fill: false,
        h_fill: false,
    }
}

fn measure_frame(
    node: &IrNode, f: &FrameData, pw: f32, ph: f32, canvas: &Canvas,
) -> MeasuredSize {
    let (mut w, wf) = dim_val(&f.width, pw);
    let (mut h, hf) = dim_val(&f.height, ph);

    // Hug: compute from children
    if w == 0.0 && !wf && !node.children.is_empty() {
        let pad = padding_vals(f);
        let (cw, ch) = measure_children(node, pw, ph, f, canvas);
        if w == 0.0 && !wf { w = cw + pad[1] + pad[3]; }
        if h == 0.0 && !hf { h = ch + pad[0] + pad[2]; }
    } else if h == 0.0 && !hf && !node.children.is_empty() {
        let pad = padding_vals(f);
        let (_, ch) = measure_children(node, pw, ph, f, canvas);
        h = ch + pad[0] + pad[2];
    }

    MeasuredSize { w, h, w_fill: wf, h_fill: hf }
}

fn measure_children(
    node: &IrNode, pw: f32, ph: f32, f: &FrameData, canvas: &Canvas,
) -> (f32, f32) {
    let layout = f.layout.as_ref();
    let is_h = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Horizontal));
    let gap = layout.and_then(|l| l.gap).unwrap_or(0.0) as f32;

    let sizes: Vec<_> = node.children.iter()
        .map(|c| measure_node(c, pw, ph, canvas))
        .collect();

    let total_gap = gap * (sizes.len().saturating_sub(1)) as f32;

    if is_h {
        let cw = sizes.iter().map(|s| s.w).sum::<f32>() + total_gap;
        let ch = sizes.iter().map(|s| s.h).fold(0.0f32, f32::max);
        (cw, ch)
    } else {
        let cw = sizes.iter().map(|s| s.w).fold(0.0f32, f32::max);
        let ch = sizes.iter().map(|s| s.h).sum::<f32>() + total_gap;
        (cw, ch)
    }
}

fn padding_vals(f: &FrameData) -> [f32; 4] {
    f.padding.as_ref().map(|p| p.0.map(|v| v as f32)).unwrap_or([0.0; 4])
}

fn layout_children(
    node: &IrNode, px: f32, py: f32, pw: f32, ph: f32,
    tree: &mut LayoutTree, canvas: &Canvas,
) {
    if node.children.is_empty() { return; }

    let (pad, layout) = match &node.data {
        IrNodeData::Frame(f) => (padding_vals(f), f.layout.as_ref()),
        _ => return,
    };

    let ix = px + pad[3];
    let iy = py + pad[0];
    let iw = pw - pad[1] - pad[3];
    let ih = ph - pad[0] - pad[2];

    let is_h = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Horizontal));
    let gap = layout.and_then(|l| l.gap).unwrap_or(0.0) as f32;
    let align = layout.and_then(|l| l.align.as_ref());
    let justify = layout.and_then(|l| l.justify.as_ref());

    // Measure children
    let mut sizes: Vec<MeasuredSize> = node.children.iter()
        .map(|c| measure_node(c, iw, ih, canvas))
        .collect();

    // Resolve fill on cross axis
    for s in &mut sizes {
        if is_h && s.h_fill { s.h = ih; }
        if !is_h && s.w_fill { s.w = iw; }
    }

    // Resolve fill on main axis
    let fixed_main: f32 = sizes.iter().map(|s| if is_h { s.w } else { s.h }).sum();
    let total_gap = gap * sizes.len().saturating_sub(1) as f32;
    let free = (if is_h { iw } else { ih }) - fixed_main - total_gap;
    let fill_count = sizes.iter().filter(|s| if is_h { s.w_fill } else { s.h_fill }).count();
    if fill_count > 0 && free > 0.0 {
        let each = free / fill_count as f32;
        for s in &mut sizes {
            if is_h && s.w_fill { s.w = each; }
            if !is_h && s.h_fill { s.h = each; }
        }
    }

    // Position
    let total: f32 = sizes.iter().map(|s| if is_h { s.w } else { s.h }).sum::<f32>() + total_gap;
    let free_space = (if is_h { iw } else { ih }) - total;

    let (mut cx, mut cy) = (ix, iy);
    let mut space_between = 0.0f32;

    match justify {
        Some(Justify::Center) => { if is_h { cx += free_space / 2.0 } else { cy += free_space / 2.0 } }
        Some(Justify::End) => { if is_h { cx += free_space } else { cy += free_space } }
        Some(Justify::SpaceBetween) if node.children.len() > 1 => {
            space_between = free_space / (node.children.len() - 1) as f32;
        }
        Some(Justify::SpaceAround) if !node.children.is_empty() => {
            let around = free_space / (node.children.len() * 2) as f32;
            if is_h { cx += around } else { cy += around }
            space_between = around;
        }
        _ => {}
    }

    for (i, child) in node.children.iter().enumerate() {
        let s = &sizes[i];
        let (mut nx, mut ny) = (cx, cy);

        // Cross-axis alignment
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
        layout_children(child, nx, ny, s.w, s.h, tree, canvas);

        let step = (if is_h { s.w } else { s.h }) + gap + space_between;
        if is_h { cx += step } else { cy += step }
    }
}

pub fn make_font(
    family: Option<&str>, weight: &Option<pastel_lang::ir::style::FontWeight>, size: f32,
) -> Font {
    let style = match weight {
        Some(w) => {
            let w_val = match w {
                pastel_lang::ir::style::FontWeight::Thin => skia_safe::font_style::Weight::THIN,
                pastel_lang::ir::style::FontWeight::Light => skia_safe::font_style::Weight::LIGHT,
                pastel_lang::ir::style::FontWeight::Normal => skia_safe::font_style::Weight::NORMAL,
                pastel_lang::ir::style::FontWeight::Medium => skia_safe::font_style::Weight::MEDIUM,
                pastel_lang::ir::style::FontWeight::Semibold => skia_safe::font_style::Weight::SEMI_BOLD,
                pastel_lang::ir::style::FontWeight::Bold => skia_safe::font_style::Weight::BOLD,
                pastel_lang::ir::style::FontWeight::Extrabold => skia_safe::font_style::Weight::EXTRA_BOLD,
                pastel_lang::ir::style::FontWeight::Black => skia_safe::font_style::Weight::BLACK,
            };
            FontStyle::new(w_val, skia_safe::font_style::Width::NORMAL, skia_safe::font_style::Slant::Upright)
        }
        None => FontStyle::normal(),
    };

    let fm = FontMgr::default();
    let typeface = fm.match_family_style(family.unwrap_or("Helvetica"), style)
        .or_else(|| fm.match_family_style("Arial", style))
        .or_else(|| fm.match_family_style("sans-serif", style))
        .expect("no font available on system");
    Font::from_typeface(typeface, size)
}
