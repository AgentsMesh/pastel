use pastel_lang::ir::node::{FrameData, IrNode};
use pastel_lang::ir::style::LayoutMode;
use skia_safe::Canvas;

use super::layout::{dim, is_absolute, measure, pad, Size};

/// Measure a frame node, handling flex and grid layouts.
pub(crate) fn measure_frame(node: &IrNode, f: &FrameData, aw: f32, ah: f32, c: &Canvas) -> Size {
    let (mut w, wf) = dim(f.width.as_ref(), aw);
    let (mut h, hf) = dim(f.height.as_ref(), ah);
    let p = pad(f);

    let inner_w = if wf || w > 0.0 {
        (if wf { aw } else { w }) - p[1] - p[3]
    } else {
        aw
    };
    let inner_h = if hf || h > 0.0 {
        (if hf { ah } else { h }) - p[0] - p[2]
    } else {
        ah
    };

    let flow: Vec<&IrNode> = node.children.iter().filter(|ch| !is_absolute(ch)).collect();
    if flow.is_empty() {
        return Size {
            w,
            h,
            w_fill: wf,
            h_fill: hf,
            baseline: 0.0,
        };
    }

    let layout = f.layout.as_ref();
    let is_grid = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Grid));

    if is_grid {
        return measure_grid(f, &flow, inner_w, inner_h, c, w, h, wf, hf, &p);
    }

    let is_h = matches!(layout.map(|l| &l.mode), Some(LayoutMode::Horizontal));
    let gap = layout.and_then(|l| l.gap).unwrap_or(0.0) as f32;

    let sizes: Vec<Size> = flow
        .iter()
        .map(|ch| measure(ch, inner_w, inner_h, c))
        .collect();

    let tg = gap * sizes.len().saturating_sub(1) as f32;
    let (cw, ch_) = if is_h {
        (
            sizes.iter().map(|s| s.w).sum::<f32>() + tg,
            sizes.iter().map(|s| s.h).fold(0.0f32, f32::max),
        )
    } else {
        (
            sizes.iter().map(|s| s.w).fold(0.0f32, f32::max),
            sizes.iter().map(|s| s.h).sum::<f32>() + tg,
        )
    };

    if w == 0.0 && !wf {
        w = cw + p[1] + p[3];
    }
    if h == 0.0 && !hf {
        h = ch_ + p[0] + p[2];
    }

    Size {
        w,
        h,
        w_fill: wf,
        h_fill: hf,
        baseline: 0.0,
    }
}

#[allow(clippy::too_many_arguments)]
fn measure_grid(
    f: &FrameData,
    flow: &[&IrNode],
    iw: f32,
    ih: f32,
    c: &Canvas,
    mut w: f32,
    mut h: f32,
    wf: bool,
    hf: bool,
    p: &[f32; 4],
) -> Size {
    let layout = f.layout.as_ref().unwrap();
    let cols = layout.columns.unwrap_or(2).max(1) as usize;
    let gap = layout.gap.unwrap_or(0.0) as f32;

    let col_w = (iw - gap * (cols as f32 - 1.0).max(0.0)) / cols as f32;
    let sizes: Vec<Size> = flow.iter().map(|ch| measure(ch, col_w, ih, c)).collect();

    let row_count = flow.len().div_ceil(cols);
    let mut row_heights = vec![0.0f32; row_count];
    for (i, s) in sizes.iter().enumerate() {
        let row = i / cols;
        row_heights[row] = row_heights[row].max(s.h);
    }
    let total_h =
        row_heights.iter().sum::<f32>() + gap * row_heights.len().saturating_sub(1) as f32;
    let total_w = iw;

    if w == 0.0 && !wf {
        w = total_w + p[1] + p[3];
    }
    if h == 0.0 && !hf {
        h = total_h + p[0] + p[2];
    }

    Size {
        w,
        h,
        w_fill: wf,
        h_fill: hf,
        baseline: 0.0,
    }
}
