//! Layout + rendering (ratatui). Keeps drawing concerns out of `app` logic.

use std::io::Write;

use crossterm::{cursor, queue, style, terminal as cterm};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Mode, SaveState};
use crate::editor::{Editor, cursor_segment_idx, display_width, wrap_line};

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    // Full-buffer reset every frame avoids ghost cells: shorter lines, closed modals, and
    // terminal selection highlighting often leave stale glyphs/attributes if we only paint deltas.
    frame.render_widget(Clear, area);

    let header_h = 4u16;
    let footer_h = 3u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_h),
            Constraint::Min(1),
            Constraint::Length(footer_h),
        ])
        .split(area);

    let main_block = Block::default().borders(Borders::NONE);
    let inner_main = main_block.inner(chunks[1]);
    app.last_main_viewport_h = inner_main.height;
    app.ensure_cursor_visible(inner_main.height, inner_main.width);

    app.editor_area = (inner_main.x, inner_main.y, inner_main.width, inner_main.height);

    render_header(frame, chunks[0], app);
    // Editor content is rendered after terminal.draw() via raw_render_editor()
    // so the terminal sees natural auto-wrapping instead of cell-positioned text.
    render_footer(frame, chunks[2], app);

    match &app.mode {
        Mode::SaveAs { input } => render_save_as_overlay(frame, inner_main, input),
        Mode::ClearConfirm => render_clear_overlay(frame, inner_main),
        Mode::Editing => {}
    }

    match &app.mode {
        Mode::Editing => {
            if let Some((cx, cy)) = cursor_xy(inner_main, app) {
                frame.set_cursor_position((cx, cy));
            }
        }
        Mode::SaveAs { .. } => {
            // Cursor position is set inside `render_save_as_overlay`.
        }
        Mode::ClearConfirm => {
            // If `set_cursor_position` is not called, ratatui keeps the cursor hidden.
        }
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let draft_short = shrink_path(&app.draft_path, area.width.saturating_sub(2) as usize);

    let status_style = match &app.save_state {
        SaveState::SaveFailed(_) => Style::default().fg(Color::Red),
        SaveState::Modified => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Green),
    };

    let status_text = match &app.save_state {
        SaveState::Saved => "Saved",
        SaveState::Modified => "Modified",
        SaveState::AutoSaved => "Auto-saved",
        SaveState::SaveFailed(msg) => msg.as_str(),
    };

    let toast = app
        .toast
        .as_ref()
        .map(|t| format!(" | {t}"))
        .unwrap_or_default();

    let line1 = Line::from(vec![
        Span::styled(
            " tmptxt ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - Temporary terminal scratchpad"),
    ]);

    let line2 = Line::from(vec![
        Span::styled("Auto-save enabled", Style::default().fg(Color::DarkGray)),
        Span::raw(" | "),
        Span::styled(status_text, status_style),
        Span::styled(toast, Style::default().fg(Color::Cyan)),
    ]);

    let line3 = Line::from(vec![
        Span::styled("Default draft: ", Style::default().fg(Color::DarkGray)),
        Span::raw(draft_short),
    ]);

    let line4 = Line::from(vec![Span::styled(
        "Not a full editor - a single auto-saving scratchpad.",
        Style::default().fg(Color::DarkGray),
    )]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let p = Paragraph::new(vec![line1, line2, line3, line4])
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(p, area);
}

fn render_footer(frame: &mut Frame, area: Rect, _app: &App) {
    let help = Line::from(vec![
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Exit  "),
        Span::styled("Ctrl+S", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Save As  "),
        Span::styled("Ctrl+L", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Clear (confirm)  "),
        Span::styled("Auto-save on", Style::default().fg(Color::DarkGray)),
    ]);

    let explain = Line::from(vec![Span::styled(
        "How to: Esc exits and saves automatically; Save As exports a copy; Clear wipes the scratchpad after confirmation.",
        Style::default().fg(Color::DarkGray),
    )]);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let p = Paragraph::new(vec![help, explain])
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(p, area);
}

/// Queues erasing a rectangle of cells on the physical terminal.  Does not flush:
/// callers flush so the clears and the writes that follow land in one pass — an
/// intermediate flush would briefly blank the region between the two (visible as
/// a flicker once content fills the screen).
///
/// Editor text is drawn outside ratatui's cell buffer (`raw_render_editor`), so
/// ratatui's diff-based rendering never sees it and cannot clear it.  Blanking the
/// physical cells an overlay covers makes that overlay's background opaque, hiding
/// only the text behind it while the rest of the editor stays visible.
pub fn clear_rect<W: Write>(out: &mut W, rect: Rect) -> std::io::Result<()> {
    for row in 0..rect.height {
        queue!(
            out,
            cursor::MoveTo(rect.x, rect.y + row),
            cterm::Clear(cterm::ClearType::UntilNewLine)
        )?;
    }
    Ok(())
}

/// Erases every row of the editor area on the physical terminal (used by the raw
/// editor renderer before repainting each frame).
pub fn clear_editor_area<W: Write>(out: &mut W, app: &App) -> std::io::Result<()> {
    let (x, y, w, h) = app.editor_area;
    if w == 0 || h == 0 {
        return Ok(());
    }
    clear_rect(out, Rect::new(x, y, w, h))
}

/// Writes editor text directly to the terminal via crossterm, bypassing ratatui's
/// cell-based rendering.  The terminal's own auto-wrap (DECAWM) marks continuation
/// rows as soft line breaks, so mouse-selection / copy never inserts spurious newlines
/// at wrap points.
pub fn raw_render_editor<W: Write>(out: &mut W, app: &mut App) -> std::io::Result<()> {
    let (x, y, w, h) = app.editor_area;
    if w == 0 || h == 0 {
        return Ok(());
    }

    // Idle frames repaint a byte-identical editor every 100 ms.  That continuous
    // full-area clear+paint is what the terminal occasionally renders mid-frame —
    // the editor is blank between the erase phase and the paint phase, so the
    // whole screen visibly blinks.  When nothing on the viewport changed, emit
    // nothing and leave the screen untouched.
    let key = crate::app::RawFrameKey {
        area: app.editor_area,
        scroll_row: app.scroll_row,
        content_rev: app.editor.rev,
    };
    if app.last_raw == Some(key) {
        return Ok(());
    }
    app.last_raw = Some(key);

    queue!(out, cursor::SavePosition)?;
    clear_editor_area(out, app)?;

    let (first_line, seg_offset) = app.editor.vrow_to_line_seg(app.scroll_row, w);
    let mut screen_y = y;
    let max_y = y + h;
    let mut line_idx = first_line;
    let mut is_first = true;

    while line_idx < app.editor.line_count() && screen_y < max_y {
        let line_text = app.editor.line(line_idx);
        let segs = wrap_line(line_text, w);

        let start_seg = if is_first { seg_offset } else { 0 };
        is_first = false;

        let total_vis_rows = segs.len() - start_seg;
        let rows_to_render = total_vis_rows.min((max_y - screen_y) as usize);

        queue!(out, cursor::MoveTo(x, screen_y))?;

        let char_start = segs[start_seg].0;
        let char_end = if start_seg + rows_to_render < segs.len() {
            segs[start_seg + rows_to_render].0
        } else {
            line_text.chars().count()
        };

        if char_end > char_start {
            let text: String = line_text
                .chars()
                .skip(char_start)
                .take(char_end - char_start)
                .collect();
            queue!(out, style::Print(text))?;
        }

        screen_y += rows_to_render as u16;
        line_idx += 1;
    }

    queue!(out, cursor::RestorePosition)?;
    out.flush()?;
    Ok(())
}

fn cursor_xy(inner: Rect, app: &App) -> Option<(u16, u16)> {
    let w = inner.width.max(1);
    let cursor_vrow = app.editor.cursor_visual_row(w);
    if cursor_vrow < app.scroll_row {
        return None;
    }
    let screen_row = cursor_vrow - app.scroll_row;
    if screen_row >= inner.height as usize {
        return None;
    }
    let y = inner.y + screen_row as u16;
    let line_text = app.editor.line(app.editor.cursor_line);
    let segs = wrap_line(line_text, w);
    let seg_idx = cursor_segment_idx(&segs, app.editor.cursor_col);
    let seg_start = segs[seg_idx].0;
    let cursor_vx = Editor::visual_width_before(line_text, app.editor.cursor_col)
        .saturating_sub(Editor::visual_width_before(line_text, seg_start));
    let x = inner.x + cursor_vx.min(w.saturating_sub(1));
    Some((x, y))
}

fn shrink_path(path: &std::path::Path, max_chars: usize) -> String {
    let s = path.display().to_string();
    if s.chars().count() <= max_chars {
        return s;
    }
    if max_chars < 3 {
        return ".".repeat(max_chars);
    }
    let suffix_len = max_chars - 3;
    let skip = s.chars().count().saturating_sub(suffix_len);
    format!("...{}", s.chars().skip(skip).collect::<String>())
}

/// Centered confirm-dialog rectangle within the editor area.
fn clear_overlay_rect(main: Rect) -> Rect {
    let w = (main.width * 4 / 5).max(40).min(main.width);
    let h = 7u16.min(main.height).max(5);
    let x = main.x + (main.width.saturating_sub(w)) / 2;
    let y = main.y + (main.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

/// Bottom save-as bar rectangle within the editor area.
fn save_as_overlay_rect(main: Rect) -> Rect {
    let h = 5u16.min(main.height.max(1));
    Rect::new(main.x, main.y + main.height.saturating_sub(h), main.width, h)
}

/// The rectangle the current overlay occupies, or an empty rect while editing.
/// Used to blank exactly the physical cells an overlay covers before drawing it.
pub fn overlay_rect(app: &App) -> Rect {
    let (x, y, w, h) = app.editor_area;
    let main = Rect::new(x, y, w, h);
    match &app.mode {
        Mode::ClearConfirm => clear_overlay_rect(main),
        Mode::SaveAs { .. } => save_as_overlay_rect(main),
        Mode::Editing => Rect::default(),
    }
}

fn render_save_as_overlay(frame: &mut Frame, main: Rect, input: &str) {
    let area = save_as_overlay_rect(main);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Save As ");
    let inner = block.inner(area);

    let text = vec![
        Line::from(Span::styled(
            "Save As - export a copy (default draft is unchanged)",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("Type a path, Enter to save, Esc to cancel.")),
        Line::from(""),
    ];

    let p = Paragraph::new(text).block(block.clone());
    frame.render_widget(p, area);

    let path_line = Line::from(vec![
        Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
        Span::raw(input),
    ]);
    let path_para = Paragraph::new(path_line).alignment(Alignment::Left);
    let path_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    frame.render_widget(path_para, path_area);

    let prefix = "Path: ";
    let cursor_x = path_area.x + unicode_display_width(prefix) + unicode_display_width(input);
    let cursor_y = path_area.y;
    frame.set_cursor_position((cursor_x.min(path_area.x + path_area.width.saturating_sub(1)), cursor_y));
}

fn unicode_display_width(s: &str) -> u16 {
    display_width(s) as u16
}

fn render_clear_overlay(frame: &mut Frame, main: Rect) {
    let area = clear_overlay_rect(main);
    frame.render_widget(Clear, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Clear all current scratchpad content?",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "This cannot be undone.",
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from("Press y to confirm, n or Esc to cancel."),
        Line::from(""),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Confirm ");
    let p = Paragraph::new(text).block(block).alignment(Alignment::Center);
    frame.render_widget(p, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A writer that records bytes and counts explicit flushes.
    struct CountingWriter {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl std::io::Write for CountingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    /// Regression test: `raw_render_editor` must clear and repaint in a single
    /// flush.  An intermediate flush (e.g. inside the clear helper) briefly blanks
    /// the whole editor area before the text is painted — a visible flicker once
    /// content fills the screen.
    #[test]
    fn raw_render_editor_paints_in_one_flush() {
        let mut app = App::new(
            PathBuf::from("/tmp/draft.txt"),
            "line one\nline two\n".to_string(),
        );
        app.editor_area = (0, 4, 40, 8);
        let mut out = CountingWriter {
            bytes: Vec::new(),
            flushes: 0,
        };
        raw_render_editor(&mut out, &mut app).unwrap();
        assert_eq!(
            out.flushes, 1,
            "clears and paint must share one flush to avoid a blank flicker"
        );
        let painted = String::from_utf8_lossy(&out.bytes);
        assert!(painted.contains("line one"), "painted text missing from output");
        assert!(painted.contains("line two"), "painted text missing from output");
    }

    /// Regression test: an idle frame whose viewport is byte-identical to the
    /// last one must emit *nothing*.  The old code re-cleared and re-painted the
    /// whole editor 10×/s; terminals occasionally render the blank moment between
    /// the erase phase and the paint phase, which is the visible screen flicker.
    #[test]
    fn raw_render_editor_skips_unchanged_idle_frames() {
        let mut app = App::new(
            PathBuf::from("/tmp/draft.txt"),
            "line one\nline two\n".to_string(),
        );
        app.editor_area = (0, 4, 40, 8);

        let mut out1 = CountingWriter {
            bytes: Vec::new(),
            flushes: 0,
        };
        raw_render_editor(&mut out1, &mut app).unwrap();
        assert!(out1.flushes == 1 && !out1.bytes.is_empty(), "first frame paints");

        // Same viewport, nothing changed: the renderer must not touch the terminal.
        let mut out2 = CountingWriter {
            bytes: Vec::new(),
            flushes: 0,
        };
        raw_render_editor(&mut out2, &mut app).unwrap();
        assert_eq!(out2.bytes, Vec::new(), "unchanged idle frame wrote bytes");
        assert_eq!(out2.flushes, 0, "unchanged idle frame flushed");
    }

    /// A content edit (rev bump) must force a repaint, even if nothing else moved.
    #[test]
    fn raw_render_editor_repaints_after_edit() {
        let mut app = App::new(
            PathBuf::from("/tmp/draft.txt"),
            "line one\nline two\n".to_string(),
        );
        app.editor_area = (0, 4, 40, 8);
        raw_render_editor(&mut CountingWriter { bytes: Vec::new(), flushes: 0 }, &mut app).unwrap();

        app.editor.insert_char('x'); // mutates content -> rev bumps

        let mut out = CountingWriter {
            bytes: Vec::new(),
            flushes: 0,
        };
        raw_render_editor(&mut out, &mut app).unwrap();
        let painted = String::from_utf8_lossy(&out.bytes);
        assert!(painted.contains("xline one"), "edited line not repainted");
    }

    /// Scrolling (scroll_row change) must force a repaint.
    #[test]
    fn raw_render_editor_repaints_after_scroll() {
        let mut app = App::new(
            PathBuf::from("/tmp/draft.txt"),
            "line one\nline two\nline three\nline four\n".to_string(),
        );
        app.editor_area = (0, 4, 40, 3);
        raw_render_editor(&mut CountingWriter { bytes: Vec::new(), flushes: 0 }, &mut app).unwrap();

        // Cursor to the last line (visual row 3) with a 3-row viewport: the
        // real loop runs ensure_cursor_visible during render, which scrolls.
        app.editor.move_down(40);
        app.editor.move_down(40);
        app.editor.move_down(40);
        app.ensure_cursor_visible(3, 40);
        assert!(app.scroll_row > 0, "expected the viewport to scroll");

        let mut out = CountingWriter {
            bytes: Vec::new(),
            flushes: 0,
        };
        raw_render_editor(&mut out, &mut app).unwrap();
        let painted = String::from_utf8_lossy(&out.bytes);
        assert!(painted.contains("line four"), "scrolled viewport not repainted");
    }

    /// An overlay erases physical editor cells; the cache must be invalidated so
    /// editing resumes with a full repaint (not a stale "unchanged" skip).
    #[test]
    fn raw_render_editor_repaints_after_overlay_clear() {
        let mut app = App::new(
            PathBuf::from("/tmp/draft.txt"),
            "line one\nline two\n".to_string(),
        );
        app.editor_area = (0, 4, 40, 8);
        raw_render_editor(&mut CountingWriter { bytes: Vec::new(), flushes: 0 }, &mut app).unwrap();

        // Overlay flow: raw_clear_pending is consumed in main.rs, which also
        // clears app.last_raw. Simulate both sides of that handshake.
        app.raw_clear_pending = true;
        app.last_raw = None;

        let mut out = CountingWriter {
            bytes: Vec::new(),
            flushes: 0,
        };
        raw_render_editor(&mut out, &mut app).unwrap();
        let painted = String::from_utf8_lossy(&out.bytes);
        assert!(painted.contains("line one"), "editor not repainted after overlay");
    }
}
