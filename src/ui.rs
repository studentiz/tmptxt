//! Layout + rendering (ratatui). Keeps drawing concerns out of `app` logic.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Mode, SaveState};
use crate::editor::{Editor, wrap_line, visual_line_count, cursor_segment_idx};

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

    render_header(frame, chunks[0], app);
    render_editor(frame, inner_main, app);
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

fn render_editor(frame: &mut Frame, inner: Rect, app: &App) {
    let w = inner.width.max(1);
    let h = inner.height as usize;
    let (mut cur_line, mut cur_seg) = vrow_to_line_seg(&app.editor, app.scroll_row, w);

    for row in 0..h {
        let y = inner.y + row as u16;
        let row_area = Rect::new(inner.x, y, w, 1);

        let content = if cur_line < app.editor.line_count() {
            let line_text = app.editor.line(cur_line);
            let segs = wrap_line(line_text, w);
            let s = if cur_seg < segs.len() {
                let (start, end) = segs[cur_seg];
                line_text.chars().skip(start).take(end - start).collect::<String>()
            } else {
                String::new()
            };
            cur_seg += 1;
            if cur_seg >= segs.len() {
                cur_line += 1;
                cur_seg = 0;
            }
            s
        } else {
            String::new()
        };

        let padded = pad_visual_width(&content, w);
        let line = Line::from(vec![Span::raw(padded)]);
        let p = Paragraph::new(line).alignment(Alignment::Left);
        frame.render_widget(p, row_area);
    }
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

/// Maps a visual-row offset to the corresponding (logical_line, segment_index).
fn vrow_to_line_seg(editor: &Editor, vrow: usize, width: u16) -> (usize, usize) {
    let mut remaining = vrow;
    for i in 0..editor.line_count() {
        let count = visual_line_count(editor.line(i), width);
        if remaining < count {
            return (i, remaining);
        }
        remaining -= count;
    }
    (editor.line_count(), 0)
}

/// Pad with ASCII spaces so the row occupies `target_width` terminal columns (for wide chars).
fn pad_visual_width(s: &str, target_width: u16) -> String {
    let mut out = String::from(s);
    let mut vis: u16 = s
        .chars()
        .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0) as u16)
        .sum();
    let tw = target_width.max(1);
    while vis < tw {
        out.push(' ');
        vis = vis.saturating_add(1);
    }
    out
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

fn render_save_as_overlay(frame: &mut Frame, main: Rect, input: &str) {
    let h = 5u16.min(main.height.max(5));
    let area = Rect::new(main.x, main.y + main.height.saturating_sub(h), main.width, h);
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
    s.chars()
        .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0) as u16)
        .sum()
}

fn render_clear_overlay(frame: &mut Frame, main: Rect) {
    let w = (main.width * 4 / 5).max(40).min(main.width);
    let h = 7u16.min(main.height).max(5);
    let x = main.x + (main.width.saturating_sub(w)) / 2;
    let y = main.y + (main.height.saturating_sub(h)) / 2;
    let area = Rect::new(x, y, w, h);

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
