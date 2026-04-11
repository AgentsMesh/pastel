use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::{Align, Justify, LayoutMode};
use skia_safe::Canvas;

use super::layout::{LayoutTree, Rect, is_absolute, measure, resolve_main, pad};

/// Place children of a frame node according to flex or grid layout.
pub(crate) fn place_children(
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

    // Place absolutely positioned children first
    for child in &node.children {
        if !is_absolute(child) { continue; }
        place_absolute(child, px, py, pw, ph, tree, c);
    }

    let flow: Vec<&IrNode> = node.children.iter()
        .filter(|ch| !is_absolute(ch)).collect();
    if flow.is_empty() { return; }

    let is_grid = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Grid));
    if is_grid {
        place_grid(&flow, layout.unwrap(), ix, iy, iw, ih, tree, c);
        return;
    }

    let is_stack = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Stack));
    if is_stack {
        place_stack(&flow, layout, ix, iy, iw, ih, tree, c);
        return;
    }

    place_flex(&flow, layout, ix, iy, iw, ih, tree, c);
}

#[allow(clippy::too_many_arguments)]
fn place_grid(
    flow: &[&IrNode], layout: &pastel_lang::ir::style::Layout,
    ix: f32, iy: f32, iw: f32, ih: f32,
    tree: &mut LayoutTree, c: &Canvas,
) {
    let cols = layout.columns.unwrap_or(2).max(1) as usize;
    let gap = layout.gap.unwrap_or(0.0) as f32;
    let col_w = (iw - gap * (cols as f32 - 1.0).max(0.0)) / cols as f32;

    let sizes: Vec<super::layout::Size> = flow.iter()
        .map(|ch| measure(ch, col_w, ih, c)).collect();

    let row_count = flow.len().div_ceil(cols);
    let mut row_heights = vec![0.0f32; row_count];
    for (i, s) in sizes.iter().enumerate() {
        let row = i / cols;
        row_heights[row] = row_heights[row].max(s.h);
    }

    let mut row_y = iy;
    for (i, child) in flow.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let nx = ix + col as f32 * (col_w + gap);
        let ny = row_y;
        let s = &sizes[i];

        tree.rects.insert(child.id.clone(), Rect { x: nx, y: ny, w: col_w, h: s.h });
        place_children(child, nx, ny, col_w, s.h, tree, c);

        if col == cols - 1 || i == flow.len() - 1 {
            row_y += row_heights[row] + gap;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn place_flex(
    flow: &[&IrNode], layout: Option<&pastel_lang::ir::style::Layout>,
    ix: f32, iy: f32, iw: f32, ih: f32,
    tree: &mut LayoutTree, c: &Canvas,
) {
    let is_h = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Horizontal));
    let gap = layout.and_then(|l| l.gap).unwrap_or(0.0) as f32;
    let align = layout.and_then(|l| l.align.as_ref());
    let justify = layout.and_then(|l| l.justify.as_ref());

    let mut sizes: Vec<super::layout::Size> = flow.iter()
        .map(|ch| measure(ch, iw, ih, c)).collect();

    for s in &mut sizes {
        if is_h && s.h_fill { s.h = ih; }
        if !is_h && s.w_fill { s.w = iw; }
    }

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

    let total: f32 = sizes.iter()
        .map(|s| if is_h { s.w } else { s.h }).sum::<f32>() + tg;
    let fs = (if is_h { iw } else { ih }) - total;

    let (mut cx, mut cy) = (ix, iy);
    let mut sb = 0.0f32;

    match justify {
        Some(Justify::Center) => { if is_h { cx += fs / 2.0 } else { cy += fs / 2.0 } }
        Some(Justify::End) => { if is_h { cx += fs } else { cy += fs } }
        Some(Justify::SpaceBetween) if flow.len() > 1 => {
            sb = fs / (flow.len() - 1) as f32;
        }
        Some(Justify::SpaceAround) if !flow.is_empty() => {
            let ar = fs / (flow.len() * 2) as f32;
            if is_h { cx += ar } else { cy += ar }
            sb = ar;
        }
        _ => {}
    }

    // For baseline alignment, compute the maximum baseline across all children.
    let max_baseline = if is_h && matches!(align, Some(Align::Baseline)) {
        sizes.iter().map(|s| s.baseline).fold(0.0f32, f32::max)
    } else { 0.0 };

    for (i, child) in flow.iter().enumerate() {
        let s = &sizes[i];
        let (mut nx, mut ny) = (cx, cy);

        if is_h {
            match align {
                Some(Align::Center) => ny = iy + (ih - s.h) / 2.0,
                Some(Align::End) => ny = iy + ih - s.h,
                Some(Align::Baseline) => {
                    // Align baselines: offset each child so its baseline matches max_baseline.
                    ny = iy + (max_baseline - s.baseline);
                }
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

/// Stack layout: all children occupy the same position (like SwiftUI ZStack).
/// Children default to filling the parent container.
#[allow(clippy::too_many_arguments)]
fn place_stack(
    flow: &[&IrNode], layout: Option<&pastel_lang::ir::style::Layout>,
    ix: f32, iy: f32, iw: f32, ih: f32,
    tree: &mut LayoutTree, c: &Canvas,
) {
    let align = layout.and_then(|l| l.align.as_ref());
    let justify = layout.and_then(|l| l.justify.as_ref());

    for child in flow {
        let _s = measure(child, iw, ih, c);
        // In stack layout, children always fill the container.
        // The child's declared width/height serves as viewBox (coordinate space),
        // not as layout size.
        let w = iw;
        let h = ih;

        // Horizontal alignment (align)
        let nx = match align {
            Some(Align::Center) => ix + (iw - w) / 2.0,
            Some(Align::End) => ix + iw - w,
            _ => ix,
        };

        // Vertical alignment (justify)
        let ny = match justify {
            Some(Justify::Center) => iy + (ih - h) / 2.0,
            Some(Justify::End) => iy + ih - h,
            _ => iy,
        };

        tree.rects.insert(child.id.clone(), Rect { x: nx, y: ny, w, h });
        place_children(child, nx, ny, w, h, tree, c);
    }
}

fn place_absolute(
    child: &IrNode, px: f32, py: f32, pw: f32, ph: f32,
    tree: &mut LayoutTree, c: &Canvas,
) {
    let pos = match &child.data {
        IrNodeData::Frame(f) => f.position.as_ref().unwrap(),
        IrNodeData::Shape(s) => s.position.as_ref().unwrap(),
        _ => return,
    };
    let size = measure(child, pw, ph, c);
    let w = resolve_main(size.w, size.w_fill, pw);
    let h = resolve_main(size.h, size.h_fill, ph);

    let x = if let Some(left) = pos.left { px + left as f32 }
    else if let Some(right) = pos.right { px + pw - w - right as f32 }
    else { px };

    let y = if let Some(top) = pos.top { py + top as f32 }
    else if let Some(bottom) = pos.bottom { py + ph - h - bottom as f32 }
    else { py };

    tree.rects.insert(child.id.clone(), Rect { x, y, w, h });
    place_children(child, x, y, w, h, tree, c);
}
