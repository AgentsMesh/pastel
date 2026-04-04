use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::{Align, Justify, LayoutMode};
use skia_safe::Canvas;

use super::layout::{LayoutTree, Rect, is_absolute, measure, resolve_main, pad};

/// Place children of a frame node according to flex layout.
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

    for (i, child) in flow.iter().enumerate() {
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

fn place_absolute(
    child: &IrNode, px: f32, py: f32, pw: f32, ph: f32,
    tree: &mut LayoutTree, c: &Canvas,
) {
    let pos = match &child.data {
        IrNodeData::Frame(f) => f.position.as_ref().unwrap(),
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
