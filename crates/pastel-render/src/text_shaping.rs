use std::collections::HashMap;

use skia_safe::{Font, FontMgr, FontStyle, TextBlob};

/// A contiguous segment of text that shares a single font.
pub struct TextRun {
    pub text: String,
    pub font: Font,
}

/// Text that has been shaped: split into runs where each run uses a font
/// that can render all of its characters. Measurement and painting share
/// the same `ShapedText` so glyph coverage decisions are made exactly once.
pub struct ShapedText {
    pub runs: Vec<TextRun>,
}

// ── Construction ────────────────────────────────────────────────────

/// Shape `text` against `primary` font, falling back per-character via
/// `FontMgr::match_family_style_character` when a glyph is missing.
pub fn shape_text(text: &str, primary: &Font, style: FontStyle, size: f32) -> ShapedText {
    if text.is_empty() {
        return ShapedText { runs: vec![TextRun { text: String::new(), font: primary.clone() }] };
    }

    let fm = FontMgr::default();
    // Cache: char → Option<Font>. Avoids repeated system-font queries for
    // characters that belong to the same Unicode block.
    let mut fallback_cache: HashMap<char, Option<Font>> = HashMap::new();

    let mut runs: Vec<TextRun> = Vec::new();
    let mut current_text = String::new();
    let mut current_is_primary = true; // tracks which font the current run uses
    let mut current_fallback: Option<Font> = None;

    for ch in text.chars() {
        let glyph = primary.unichar_to_glyph(ch as i32);
        if glyph != 0 {
            // Primary font can render this character.
            if !current_is_primary && !current_text.is_empty() {
                // Flush the fallback run.
                let font = current_fallback.take().unwrap_or_else(|| primary.clone());
                runs.push(TextRun { text: std::mem::take(&mut current_text), font });
            }
            current_is_primary = true;
            current_text.push(ch);
        } else {
            // Primary font lacks a glyph — try fallback.
            let fb = fallback_cache.entry(ch).or_insert_with(|| {
                resolve_fallback(&fm, style, size, ch)
            });

            if let Some(fb_font) = fb {
                // We have a fallback font for this character.
                if current_is_primary && !current_text.is_empty() {
                    // Flush the primary run.
                    runs.push(TextRun { text: std::mem::take(&mut current_text), font: primary.clone() });
                }
                // Check if this fallback font is the "same" as the current fallback run.
                let same_fallback = !current_is_primary
                    && current_fallback.as_ref().map_or(false, |cf| same_typeface(cf, fb_font));

                if current_is_primary || !same_fallback {
                    if !current_is_primary && !current_text.is_empty() {
                        let font = current_fallback.take().unwrap_or_else(|| primary.clone());
                        runs.push(TextRun { text: std::mem::take(&mut current_text), font });
                    }
                    current_fallback = Some(fb_font.clone());
                }
                current_is_primary = false;
                current_text.push(ch);
            } else {
                // No fallback found — render with primary (will show .notdef / □).
                if !current_is_primary && !current_text.is_empty() {
                    let font = current_fallback.take().unwrap_or_else(|| primary.clone());
                    runs.push(TextRun { text: std::mem::take(&mut current_text), font });
                }
                current_is_primary = true;
                current_text.push(ch);
            }
        }
    }

    // Flush remaining.
    if !current_text.is_empty() {
        let font = if current_is_primary {
            primary.clone()
        } else {
            current_fallback.unwrap_or_else(|| primary.clone())
        };
        runs.push(TextRun { text: current_text, font });
    }

    if runs.is_empty() {
        runs.push(TextRun { text: String::new(), font: primary.clone() });
    }

    ShapedText { runs }
}

// ── Measurement ─────────────────────────────────────────────────────

impl ShapedText {
    /// Total width of all runs (excluding letter-spacing).
    pub fn measure_width(&self) -> f32 {
        self.runs.iter().map(|r| {
            if r.text.is_empty() { return 0.0; }
            let (w, _) = r.font.measure_str(&r.text, None);
            w
        }).sum()
    }

    /// Total width including letter-spacing between every pair of adjacent characters.
    pub fn measure_width_with_spacing(&self, spacing: f32) -> f32 {
        let base = self.measure_width();
        let cc = self.char_count();
        base + spacing * (cc as f32 - 1.0).max(0.0)
    }

    /// Total character count across all runs.
    pub fn char_count(&self) -> usize {
        self.runs.iter().map(|r| r.text.chars().count()).sum()
    }

    /// Concatenated text content.
    pub fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }

    /// Get font metrics from the primary (first) run.
    pub fn primary_metrics(&self) -> skia_safe::FontMetrics {
        self.runs.first().map(|r| r.font.metrics().1).unwrap_or_else(|| {
            let f = Font::default();
            f.metrics().1
        })
    }
}

// ── Word Wrapping ───────────────────────────────────────────────────

/// Break text into wrapped lines, each represented as a `ShapedText`.
/// Uses greedy word-boundary wrapping (same algorithm as the original
/// `word_wrap_lines` but font-fallback-aware).
pub fn wrap_shaped_lines(
    text: &str, primary: &Font, style: FontStyle, size: f32,
    max_w: f32, spacing: f32,
) -> Vec<ShapedText> {
    let mut result: Vec<ShapedText> = Vec::new();

    for paragraph in text.split('\n') {
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        if words.is_empty() {
            result.push(shape_text("", primary, style, size));
            continue;
        }

        let mut line = String::new();
        for word in &words {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", line, word)
            };
            let shaped = shape_text(&candidate, primary, style, size);
            let w = shaped.measure_width_with_spacing(spacing);
            if w > max_w && !line.is_empty() {
                // Push current line and start new one.
                result.push(shape_text(&line, primary, style, size));
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            result.push(shape_text(&line, primary, style, size));
        }
    }

    if result.is_empty() {
        result.push(shape_text("", primary, style, size));
    }
    result
}

// ── Drawing helpers ─────────────────────────────────────────────────

impl ShapedText {
    /// Draw all runs sequentially (no letter-spacing).
    pub fn draw(&self, canvas: &skia_safe::Canvas, paint: &skia_safe::Paint, mut x: f32, y: f32) {
        for run in &self.runs {
            if run.text.is_empty() { continue; }
            if let Some(blob) = TextBlob::from_str(&run.text, &run.font) {
                canvas.draw_text_blob(&blob, (x, y), paint);
            }
            let (w, _) = run.font.measure_str(&run.text, None);
            x += w;
        }
    }

    /// Draw all runs with letter-spacing (per-character).
    pub fn draw_spaced(
        &self, canvas: &skia_safe::Canvas, paint: &skia_safe::Paint,
        mut x: f32, y: f32, spacing: f32,
    ) {
        for run in &self.runs {
            for ch in run.text.chars() {
                let s = ch.to_string();
                if let Some(blob) = TextBlob::from_str(&s, &run.font) {
                    canvas.draw_text_blob(&blob, (x, y), paint);
                }
                let (cw, _) = run.font.measure_str(&s, None);
                x += cw + spacing;
            }
        }
    }
}

// ── Internal helpers ────────────────────────────────────────────────

static BCP47_TAGS: &[&str] = &["zh-Hans", "zh-Hant", "ja", "ko", "ar", "th", "hi", "he"];

fn resolve_fallback(fm: &FontMgr, style: FontStyle, size: f32, ch: char) -> Option<Font> {
    fm.match_family_style_character("", style, BCP47_TAGS, ch as i32)
        .map(|tf| Font::from_typeface(tf, size))
}

fn same_typeface(a: &Font, b: &Font) -> bool {
    a.typeface().family_name() == b.typeface().family_name()
}
