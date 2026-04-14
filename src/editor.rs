//! UTF-8 line buffer with a cursor (character indices, not grapheme clusters).

use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone)]
pub struct Editor {
    lines: Vec<String>,
    /// Zero-based line index.
    pub cursor_line: usize,
    /// Column in Unicode scalar values (Rust `char`) before cursor.
    pub cursor_col: usize,
    preferred_col: usize,
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
            preferred_col: 0,
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
        self.preferred_col = self.cursor_col;
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
        self.preferred_col = 0;
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
            self.preferred_col = self.cursor_col;
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
        self.preferred_col = self.cursor_col;
    }

    pub fn delete_forward(&mut self) {
        let line_len = char_len(self.lines[self.cursor_line].as_str());
        if self.cursor_col < line_len {
            let line = &mut self.lines[self.cursor_line];
            let start = byte_idx_for_char_col(line, self.cursor_col);
            let ch = line[start..].chars().next().unwrap();
            let end = start + ch.len_utf8();
            line.drain(start..end);
            return;
        }

        if self.cursor_line + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.preferred_col = self.cursor_col;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = char_len(self.lines[self.cursor_line].as_str());
            self.preferred_col = self.cursor_col;
        }
    }

    pub fn move_right(&mut self) {
        let line_len = char_len(self.lines[self.cursor_line].as_str());
        if self.cursor_col < line_len {
            self.cursor_col += 1;
            self.preferred_col = self.cursor_col;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.preferred_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_line == 0 {
            return;
        }
        self.cursor_line -= 1;
        let max_col = char_len(self.lines[self.cursor_line].as_str());
        self.cursor_col = self.preferred_col.min(max_col);
    }

    pub fn move_down(&mut self) {
        if self.cursor_line + 1 >= self.lines.len() {
            return;
        }
        self.cursor_line += 1;
        let max_col = char_len(self.lines[self.cursor_line].as_str());
        self.cursor_col = self.preferred_col.min(max_col);
    }

    pub fn home(&mut self) {
        self.cursor_col = 0;
        self.preferred_col = 0;
    }

    pub fn end(&mut self) {
        self.cursor_col = char_len(self.lines[self.cursor_line].as_str());
        self.preferred_col = self.cursor_col;
    }

    pub fn page_up(&mut self, delta_lines: usize) {
        let new_line = self.cursor_line.saturating_sub(delta_lines.max(1));
        if new_line != self.cursor_line {
            self.cursor_line = new_line;
            let max_col = char_len(self.lines[self.cursor_line].as_str());
            self.cursor_col = self.preferred_col.min(max_col);
        }
    }

    pub fn page_down(&mut self, delta_lines: usize) {
        let new_line = (self.cursor_line + delta_lines.max(1)).min(self.lines.len().saturating_sub(1));
        if new_line != self.cursor_line {
            self.cursor_line = new_line;
            let max_col = char_len(self.lines[self.cursor_line].as_str());
            self.cursor_col = self.preferred_col.min(max_col);
        }
    }

    /// Clears all content to a single empty line and resets the cursor.
    pub fn clear_all(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.preferred_col = 0;
    }

    /// Visual width from start of line up to (but not including) `char_col`.
    pub fn visual_width_before(line: &str, char_col: usize) -> u16 {
        line.chars()
            .take(char_col)
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0) as u16)
            .sum()
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
}

/// Splits a logical line into visual-row segments for soft wrapping.
/// Each element is `(char_start, char_end)` — a half-open range of `char` indices.
pub fn wrap_line(line: &str, width: u16) -> Vec<(usize, usize)> {
    let w = width.max(1) as usize;
    if line.is_empty() {
        return vec![(0, 0)];
    }
    let mut segments = Vec::new();
    let mut seg_start = 0usize;
    let mut seg_width = 0usize;
    for (char_idx, c) in line.chars().enumerate() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if seg_width + cw > w && seg_width > 0 {
            segments.push((seg_start, char_idx));
            seg_start = char_idx;
            seg_width = 0;
        }
        seg_width += cw;
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
