//! UTF-8 line buffer with a cursor (character indices, not grapheme clusters).
//!
//! Long lines are soft-wrapped into visual rows, and vertical navigation moves
//! between visual rows, so Up/Down keep working on wrapped lines.
//!
//! Display widths are measured per grapheme cluster the way terminals render
//! them (emoji clusters, flags, variation selectors), so the cursor position
//! and wrap points stay aligned with the terminal's own rendering of the text.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone)]
pub struct Editor {
    lines: Vec<String>,
    /// Zero-based line index.
    pub cursor_line: usize,
    /// Column in Unicode scalar values (Rust `char`) before cursor.
    pub cursor_col: usize,
    /// Preferred visual column (display cells) to preserve across vertical
    /// moves. `None` means "not established since the last horizontal edit" —
    /// the next vertical move derives it from the cursor's current position.
    preferred_vcol: Option<usize>,
}

impl Editor {
    pub fn from_text(text: &str) -> Self {
        let mut lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|l| l.to_string()).collect()
        };
        if lines.is_empty() {
            lines.push(String::new());
        } else if text.ends_with('\n') {
            lines.push(String::new());
        }

        Self {
            lines,
            cursor_line: 0,
            cursor_col: 0,
            preferred_vcol: None,
        }
    }

    pub fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, idx: usize) -> &str {
        &self.lines[idx]
    }

    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_line];
        let byte = byte_idx_for_char_col(line, self.cursor_col);
        line.insert(byte, c);
        self.cursor_col += 1;
        self.preferred_vcol = None;
    }

    /// Inserts pasted or typed text; respects embedded newlines.
    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        let parts: Vec<&str> = s.split('\n').collect();
        if parts.len() == 1 {
            for c in parts[0].chars() {
                self.insert_char(c);
            }
            return;
        }

        for (i, part) in parts.iter().enumerate() {
            for c in part.chars() {
                self.insert_char(c);
            }
            if i + 1 < parts.len() {
                self.new_line();
            }
        }
    }

    pub fn new_line(&mut self) {
        let line = self.lines[self.cursor_line].clone();
        let before: String = line.chars().take(self.cursor_col).collect();
        let after: String = line.chars().skip(self.cursor_col).collect();
        self.lines[self.cursor_line] = before;
        self.lines.insert(self.cursor_line + 1, after);
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.preferred_vcol = None;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_line];
            let col = self.cursor_col - 1;
            let start = byte_idx_for_char_col(line, col);
            let ch = line[start..].chars().next().unwrap();
            let end = start + ch.len_utf8();
            line.drain(start..end);
            self.cursor_col = col;
            self.preferred_vcol = None;
            return;
        }

        if self.cursor_line == 0 {
            return;
        }

        let cur = self.lines.remove(self.cursor_line);
        self.cursor_line -= 1;
        let prev_len = char_len(self.lines[self.cursor_line].as_str());
        self.lines[self.cursor_line].push_str(&cur);
        self.cursor_col = prev_len;
        self.preferred_vcol = None;
    }

    pub fn delete_forward(&mut self) {
        let line_len = char_len(self.lines[self.cursor_line].as_str());
        if self.cursor_col < line_len {
            let line = &mut self.lines[self.cursor_line];
            let start = byte_idx_for_char_col(line, self.cursor_col);
            let ch = line[start..].chars().next().unwrap();
            let end = start + ch.len_utf8();
            line.drain(start..end);
            self.preferred_vcol = None;
            return;
        }

        if self.cursor_line + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next);
            self.preferred_vcol = None;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.preferred_vcol = None;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = char_len(self.lines[self.cursor_line].as_str());
            self.preferred_vcol = None;
        }
    }

    pub fn move_right(&mut self) {
        let line_len = char_len(self.lines[self.cursor_line].as_str());
        if self.cursor_col < line_len {
            self.cursor_col += 1;
            self.preferred_vcol = None;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.preferred_vcol = None;
        }
    }

    /// Moves the cursor up one *visual* row (a soft-wrapped line may occupy
    /// several rows). The preferred visual column is preserved across moves.
    pub fn move_up(&mut self, width: u16) {
        let vrow = self.cursor_visual_row(width);
        if vrow == 0 {
            return;
        }
        self.jump_to_visual_row(vrow - 1, width);
    }

    /// Moves the cursor down one *visual* row.
    pub fn move_down(&mut self, width: u16) {
        let vrow = self.cursor_visual_row(width);
        let max_vrow = self.total_visual_rows(width).saturating_sub(1);
        if vrow >= max_vrow {
            return;
        }
        self.jump_to_visual_row(vrow + 1, width);
    }

    pub fn home(&mut self) {
        self.cursor_col = 0;
        self.preferred_vcol = None;
    }

    pub fn end(&mut self) {
        self.cursor_col = char_len(self.lines[self.cursor_line].as_str());
        self.preferred_vcol = None;
    }

    pub fn page_up(&mut self, delta_rows: usize, width: u16) {
        let vrow = self.cursor_visual_row(width);
        if vrow == 0 {
            return;
        }
        self.jump_to_visual_row(vrow.saturating_sub(delta_rows.max(1)), width);
    }

    pub fn page_down(&mut self, delta_rows: usize, width: u16) {
        let vrow = self.cursor_visual_row(width);
        let max_vrow = self.total_visual_rows(width).saturating_sub(1);
        if vrow >= max_vrow {
            return;
        }
        self.jump_to_visual_row((vrow + delta_rows.max(1)).min(max_vrow), width);
    }

    /// Clears all content to a single empty line and resets the cursor.
    pub fn clear_all(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.preferred_vcol = None;
    }

    /// Moves the cursor to a specific visual row, keeping the preferred column.
    fn jump_to_visual_row(&mut self, vrow: usize, width: u16) {
        if self.preferred_vcol.is_none() {
            self.preferred_vcol = Some(self.cursor_visual_col(width));
        }
        let pref = self.preferred_vcol.unwrap_or(0);
        let (line, seg) = self.vrow_to_line_seg(vrow, width.max(1));
        self.cursor_line = line;
        let segs = wrap_line(self.line(line), width);
        let seg = seg.min(segs.len().saturating_sub(1));
        self.cursor_col = char_col_at_visual(&segs, self.line(line), seg, pref);
    }

    /// The cursor's visual column (display cells) within its current visual row.
    pub fn cursor_visual_col(&self, width: u16) -> usize {
        let line = self.line(self.cursor_line);
        let segs = wrap_line(line, width);
        let seg = cursor_segment_idx(&segs, self.cursor_col);
        let (start, _) = segs[seg];
        Self::visual_width_before(line, self.cursor_col)
            .saturating_sub(Self::visual_width_before(line, start))
            as usize
    }

    /// Absolute visual row (from top of document) where the cursor currently sits.
    pub fn cursor_visual_row(&self, width: u16) -> usize {
        let mut vrow = 0;
        for i in 0..self.cursor_line {
            vrow += visual_line_count(&self.lines[i], width);
        }
        let segs = wrap_line(&self.lines[self.cursor_line], width);
        vrow + cursor_segment_idx(&segs, self.cursor_col)
    }

    /// Maps a visual-row offset to the `(line, segment)` it falls in.
    pub fn vrow_to_line_seg(&self, vrow: usize, width: u16) -> (usize, usize) {
        let mut remaining = vrow;
        for i in 0..self.line_count() {
            let count = visual_line_count(self.line(i), width);
            if remaining < count {
                return (i, remaining);
            }
            remaining -= count;
        }
        (self.line_count(), 0)
    }

    /// Total number of visual rows in the whole document at this width.
    pub fn total_visual_rows(&self, width: u16) -> usize {
        (0..self.line_count())
            .map(|i| visual_line_count(self.line(i), width))
            .sum()
    }

    /// Visual width from start of line up to (but not including) `char_col`.
    ///
    /// Measured in grapheme clusters: if `char_col` splits a cluster, the whole
    /// cluster counts, because that is how far the terminal's text reaches.
    pub fn visual_width_before(line: &str, char_col: usize) -> u16 {
        if char_col == 0 {
            return 0;
        }
        let mut width = 0usize;
        let mut chars_seen = 0usize;
        for cluster in line.graphemes(true) {
            let clen = cluster.chars().count();
            if chars_seen + clen > char_col {
                width += cluster_width(cluster);
                break;
            }
            width += cluster_width(cluster);
            chars_seen += clen;
            if chars_seen >= char_col {
                break;
            }
        }
        width as u16
    }
}

/// Splits a logical line into visual-row segments for soft wrapping.
/// Each element is `(char_start, char_end)` — a half-open range of `char`
/// indices. Segments only break between grapheme clusters, never inside one.
pub fn wrap_line(line: &str, width: u16) -> Vec<(usize, usize)> {
    let w = width.max(1) as usize;
    if line.is_empty() {
        return vec![(0, 0)];
    }
    let mut segments = Vec::new();
    let mut seg_start = 0usize;
    let mut seg_width = 0usize;
    let mut char_idx = 0usize;
    for cluster in line.graphemes(true) {
        let cw = cluster_width(cluster);
        if seg_width + cw > w && seg_width > 0 {
            segments.push((seg_start, char_idx));
            seg_start = char_idx;
            seg_width = 0;
        }
        seg_width += cw;
        char_idx += cluster.chars().count();
    }
    segments.push((seg_start, line.chars().count()));
    segments
}

pub fn visual_line_count(line: &str, width: u16) -> usize {
    wrap_line(line, width).len()
}

/// Returns which segment index `cursor_col` falls into.
pub fn cursor_segment_idx(segs: &[(usize, usize)], cursor_col: usize) -> usize {
    for (i, &(start, end)) in segs.iter().enumerate() {
        if cursor_col >= start && cursor_col < end {
            return i;
        }
    }
    segs.len().saturating_sub(1)
}

/// Display-cell width of a whole string, matching terminal rendering.
pub fn display_width(s: &str) -> usize {
    s.graphemes(true).map(cluster_width).sum()
}

/// Returns the char column inside `line` at visual segment `seg` whose cell
/// position is closest to `vcol` (cells from the start of the segment).
///
/// Iterates the whole line but only counts cell width for clusters inside
/// `[start, end)`; `chars_seen` tracks each cluster's actual char index so the
/// pre-segment clusters are truly skipped, not just offset.
fn char_col_at_visual(segs: &[(usize, usize)], line: &str, seg: usize, vcol: usize) -> usize {
    let (start, end) = segs[seg];
    if end <= start {
        return start;
    }
    let mut cell = 0usize;
    let mut chars_seen = 0usize;
    for cluster in line.graphemes(true) {
        let clen = cluster.chars().count();
        if chars_seen >= end {
            break;
        }
        if chars_seen < start {
            chars_seen += clen;
            continue;
        }
        let cw = cluster_width(cluster);
        if cell + cw > vcol {
            break;
        }
        cell += cw;
        chars_seen += clen;
    }
    chars_seen.min(end)
}

/// Display-cell width of one grapheme cluster as terminals render it.
///
/// The per-`char` East Asian Width tables (`unicode-width`) measure each code
/// point in isolation, but terminals group emoji into clusters first (skin
/// tones, ZWJ sequences, flags, variation selectors) and give the cluster a
/// single width. Summing per-`char` widths for those drifts the cursor by an
/// extra cell or two, which is what made CJK+emoji lines misalign.
fn cluster_width(cluster: &str) -> usize {
    let chars: Vec<char> = cluster.chars().collect();
    if chars.is_empty() {
        return 0;
    }
    let base = chars[0];

    // A flag (two regional indicators) renders as one 2-cell glyph.
    if chars.len() == 2 && is_regional_indicator(base) && is_regional_indicator(chars[1]) {
        return 2;
    }

    let rest = &chars[1..];
    let base_w = UnicodeWidthChar::width(base).unwrap_or(0);

    // A variation selector (VS16) promotes the base to emoji presentation
    // (❤ → ❤️), widening a narrow symbol from 1 to 2 cells.
    if rest.contains(&'\u{FE0F}') {
        return 2;
    }
    // A wide base (emoji or CJK) plus a skin tone / ZWJ is one glyph; the
    // modifiers add no cells of their own. CJK is excluded from the ZWJ case:
    // a CJK ideograph keeps its own 2 cells even if joined by a ZWJ.
    if base_w >= 2
        && (rest.iter().any(|&c| is_skin_tone(c))
            || (rest.contains(&'\u{200D}') && !is_cjk(base)))
    {
        return 2;
    }
    // An unpaired skin-tone modifier still renders as its own 2-cell swatch.
    if is_skin_tone(base) && rest.is_empty() {
        return 2;
    }

    // Text cluster (CJK, Latin + combining marks, text-presentation symbols):
    // sum per-char widths; combining marks / ZWJ / VS contribute 0.
    chars
        .iter()
        .map(|&c| UnicodeWidthChar::width(c).unwrap_or(0))
        .sum()
}

fn is_regional_indicator(c: char) -> bool {
    (0x1F1E6..=0x1F1FF).contains(&(c as u32))
}

fn is_skin_tone(c: char) -> bool {
    (0x1F3FB..=0x1F3FF).contains(&(c as u32))
}

/// CJK ideographs render 2 cells each; a ZWJ between them does not merge them
/// into one glyph the way it does for emoji.
fn is_cjk(c: char) -> bool {
    let cp = c as u32;
    (0x3400..=0x4DBF).contains(&cp) // Extension A
        || (0x4E00..=0x9FFF).contains(&cp) // Unified
        || (0xF900..=0xFAFF).contains(&cp) // Compatibility
        || (0x20000..=0x2A6DF).contains(&cp) // Extension B
        || (0x2A700..=0x2EBEF).contains(&cp) // Extensions C–F
        || (0x30000..=0x323AF).contains(&cp) // Extension G
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn byte_idx_for_char_col(s: &str, char_col: usize) -> usize {
    if char_col == 0 {
        return 0;
    }
    match s.char_indices().nth(char_col) {
        Some((idx, _)) => idx,
        None => s.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_up_walks_visual_rows_of_a_wrapped_line() {
        // "abcdefghij" at width 5 wraps as "abcde" / "fghij".
        let mut ed = Editor::from_text("abcdefghij");
        // Cursor after 'g' (char col 6) is on the 2nd visual row, visual col 1.
        ed.cursor_col = 6;
        ed.move_up(5);
        assert_eq!(ed.cursor_line, 0, "stays on the same logical line");
        assert_eq!(ed.cursor_col, 1, "moves to the same visual column of the row above");
    }

    #[test]
    fn move_down_walks_visual_rows_of_a_wrapped_line() {
        let mut ed = Editor::from_text("abcdefghij");
        // Cursor after 'c' (char col 3) is on the 1st visual row, visual col 3.
        ed.cursor_col = 3;
        ed.move_down(5);
        assert_eq!(ed.cursor_line, 0, "stays on the same logical line");
        assert_eq!(ed.cursor_col, 8, "lands on 'h' — visual col 3 of the row below");
    }

    #[test]
    fn move_up_no_op_at_document_top() {
        // Cursor on the very first visual row (char col 2 < wrap width) — Up is a no-op.
        let mut ed = Editor::from_text("abcdefghij");
        ed.cursor_col = 2;
        ed.move_up(5);
        assert_eq!((ed.cursor_line, ed.cursor_col), (0, 2));
    }

    #[test]
    fn move_down_no_op_at_document_bottom() {
        let mut ed = Editor::from_text("abc\ndefghij");
        // Last logical line wraps to two rows at width 5; park at the very end.
        ed.cursor_line = 1;
        ed.cursor_col = 7;
        ed.move_down(5);
        assert_eq!((ed.cursor_line, ed.cursor_col), (1, 7));
    }

    #[test]
    fn move_up_crosses_wrap_boundary_between_logical_lines() {
        // Line 0: "abc" (1 row). Line 1: "defghi" (2 rows at width 4).
        let mut ed = Editor::from_text("abc\ndefghi");
        // Cursor on line 1, 1st visual row, visual col 2 (after 'e').
        ed.cursor_line = 1;
        ed.cursor_col = 2;
        ed.move_up(4);
        assert_eq!(ed.cursor_line, 0, "first Up exits the wrapped line");
        assert_eq!(ed.cursor_col, 2, "clamped to line 0's width");
        // And Down from line 0 returns to the same wrapped row.
        ed.move_down(4);
        assert_eq!((ed.cursor_line, ed.cursor_col), (1, 2));
    }

    #[test]
    fn move_up_within_wrapped_line_before_leaving() {
        let mut ed = Editor::from_text("abc\ndefghi");
        // Cursor on line 1, 2nd visual row, visual col 1 (after 'g').
        ed.cursor_line = 1;
        ed.cursor_col = 5;
        ed.move_up(4);
        // Should stay inside line 1, on its 1st visual row, visual col 1.
        assert_eq!(ed.cursor_line, 1);
        assert_eq!(ed.cursor_col, 1);
    }

    #[test]
    fn emoji_clusters_measure_terminal_width() {
        // Skin-tone modifier and ZWJ family are single 2-cell glyphs.
        assert_eq!(display_width("\u{1F44D}\u{1F3FD}"), 2, "👍🏽");
        assert_eq!(display_width("\u{1F468}\u{200D}\u{1F469}"), 2, "👨‍👩");
        assert_eq!(display_width("\u{1F1E8}\u{1F1F3}"), 2, "🇨🇳");
        assert_eq!(display_width("\u{2764}\u{FE0F}"), 2, "❤️ (VS16)");
        assert_eq!(display_width("\u{2764}"), 1, "❤ (text presentation)");
        assert_eq!(display_width("中"), 2);
        assert_eq!(display_width("ab中"), 4);
        assert_eq!(display_width("e\u{0301}"), 1, "é (combining accent)");
    }

    #[test]
    fn move_down_into_wrapped_segment_with_mixed_widths() {
        // "abc中x" at width 4 wraps as "abc" / "中x". Cursor after 'c' (char col 2)
        // is on row 0; Down must land after 中 on row 1 — char col 4, not 5.
        let mut ed = Editor::from_text("abc中x");
        ed.cursor_col = 2;
        ed.move_down(4);
        assert_eq!((ed.cursor_line, ed.cursor_col), (0, 4));
    }

    #[test]
    fn move_up_into_wrapped_segment_with_mixed_widths() {
        // line0 "abc中x" wraps as "abc" / "中x", line1 "y". Cursor at the end of
        // "y" has visual col 1; Up crosses into line0's 2nd row, which starts
        // with a 2-cell 中 — so it lands at char col 3, before the 中.
        let mut ed = Editor::from_text("abc中x\ny");
        ed.cursor_line = 1;
        ed.cursor_col = 1;
        ed.move_up(4);
        assert_eq!((ed.cursor_line, ed.cursor_col), (0, 3));
    }

    #[test]
    fn wrap_line_keeps_emoji_clusters_together() {
        // Widths: 👍🏽=2, 中=2, a=1 → wraps to ["👍🏽", "中a"] at width 3.
        let segs = wrap_line("\u{1F44D}\u{1F3FD}中a", 3);
        assert_eq!(segs, vec![(0, 2), (2, 4)], "cluster must not be split");
    }

    #[test]
    fn cursor_visual_col_tracks_emoji_width() {
        // "中👍🏽a": 中=2, 👍🏽=2, a=1. Cursor after the emoji (char col 3)
        // sits at visual column 4 — the per-char width sum would overcount the
        // skin tone and report 6.
        let mut ed = Editor::from_text("中\u{1F44D}\u{1F3FD}a");
        ed.cursor_col = 3;
        assert_eq!(ed.cursor_visual_col(10), 4);
    }
}
