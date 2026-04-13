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
