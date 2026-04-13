//! Keyboard routing for normal editing and modal flows.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Mode, SaveState};
use crate::storage::Storage;

pub fn handle_key(app: &mut App, key: KeyEvent, storage: &Storage) -> Result<(), String> {
    match std::mem::replace(&mut app.mode, Mode::Editing) {
        Mode::SaveAs { mut input } => {
            match key.code {
                KeyCode::Esc => {
                    app.mode = Mode::Editing;
                }
                KeyCode::Enter => {
                    let path = input.trim();
                    if path.is_empty() {
                        app.save_state = SaveState::SaveFailed("empty path".to_string());
                        app.mode = Mode::Editing;
                        return Ok(());
                    }
                    let path_buf = PathBuf::from(path);
                    let text = app.editor.to_text();
                    match storage.save_as(&path_buf, &text) {
                        Ok(()) => {
                            if app.dirty {
                                app.save_state = SaveState::Modified;
                            } else {
                                app.save_state = SaveState::Saved;
                            }
                            app.toast = Some(format!("Exported to {}", path_buf.display()));
                            app.mode = Mode::Editing;
                        }
                        Err(e) => {
                            app.save_state = SaveState::SaveFailed(e);
                            app.mode = Mode::Editing;
                        }
                    }
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.push(c);
                    app.mode = Mode::SaveAs { input };
                }
                KeyCode::Backspace => {
                    input.pop();
                    app.mode = Mode::SaveAs { input };
                }
                KeyCode::Delete => {
                    input.clear();
                    app.mode = Mode::SaveAs { input };
                }
                _ => {
                    app.mode = Mode::SaveAs { input };
                }
            }
            Ok(())
        }
        Mode::ClearConfirm => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    app.clear_to_empty();
                    match app.flush_draft(storage, true) {
                        Ok(()) => {
                            app.toast = Some("Cleared".to_string());
                        }
                        Err(_) => {
                            app.toast = Some("Cleared (save failed — see status)".to_string());
                        }
                    }
                    app.mode = Mode::Editing;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    app.toast = Some("Clear canceled".to_string());
                    app.mode = Mode::Editing;
                }
                _ => {
                    app.mode = Mode::ClearConfirm;
                }
            }
            Ok(())
        }
        Mode::Editing => {
            app.mode = Mode::Editing;
            handle_editing(app, key, storage)
        }
    }
}

fn handle_editing(app: &mut App, key: KeyEvent, storage: &Storage) -> Result<(), String> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('x') | KeyCode::Char('X') => {
                app.flush_draft(storage, true)?;
                app.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                app.mode = Mode::SaveAs {
                    input: String::new(),
                };
                return Ok(());
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                app.mode = Mode::ClearConfirm;
                return Ok(());
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char(c) => {
            app.editor.insert_char(c);
            app.mark_dirty();
        }
        KeyCode::Enter => {
            app.editor.new_line();
            app.mark_dirty();
        }
        KeyCode::Backspace => {
            app.editor.backspace();
            app.mark_dirty();
        }
        KeyCode::Delete => {
            app.editor.delete_forward();
            app.mark_dirty();
        }
        KeyCode::Left => {
            app.editor.move_left();
        }
        KeyCode::Right => {
            app.editor.move_right();
        }
        KeyCode::Up => {
            app.editor.move_up();
        }
        KeyCode::Down => {
            app.editor.move_down();
        }
        KeyCode::Home => {
            app.editor.home();
        }
        KeyCode::End => {
            app.editor.end();
        }
        KeyCode::PageUp => {
            let step = app.last_main_viewport_h.max(1) as usize;
            app.editor.page_up(step);
        }
        KeyCode::PageDown => {
            let step = app.last_main_viewport_h.max(1) as usize;
            app.editor.page_down(step);
        }
        _ => {}
    }

    Ok(())
}
