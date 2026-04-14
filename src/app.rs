//! Application state, autosave cadence, and draft bookkeeping.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::editor::Editor;
use crate::storage::Storage;

const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Editing,
    SaveAs { input: String },
    ClearConfirm,
}

impl Mode {
    pub fn is_editing(&self) -> bool {
        matches!(self, Mode::Editing)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveState {
    Saved,
    Modified,
    AutoSaved,
    SaveFailed(String),
}

pub struct App {
    pub editor: Editor,
    pub mode: Mode,
    pub draft_path: PathBuf,
    pub dirty: bool,
    pub save_state: SaveState,
    pub scroll_row: usize,
    /// Last rendered main editor height (for PageUp/PageDown).
    pub last_main_viewport_h: u16,
    /// Short, non-modal hint (e.g. clear/export feedback).
    pub toast: Option<String>,
    /// Last moment the buffer became dirty; used for debounced autosave.
    dirty_since: Option<Instant>,
    pub should_quit: bool,
}

impl App {
    pub fn new(draft_path: PathBuf, initial_text: String) -> Self {
        Self {
            editor: Editor::from_text(&initial_text),
            mode: Mode::Editing,
            draft_path,
            dirty: false,
            save_state: SaveState::Saved,
            scroll_row: 0,
            last_main_viewport_h: 10,
            toast: None,
            dirty_since: None,
            should_quit: false,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.toast = None;
        if !self.dirty {
            self.dirty = true;
            self.save_state = SaveState::Modified;
        }
        self.dirty_since = Some(Instant::now());
    }

    pub fn clear_to_empty(&mut self) {
        self.editor.clear_all();
        self.scroll_row = 0;
        self.mark_dirty();
    }

    /// Periodic autosave: dirty flag + timer (does not write on every keystroke).
    pub fn tick_autosave(&mut self, storage: &Storage) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        let since = match self.dirty_since {
            Some(t) => t,
            None => return Ok(()),
        };
        if since.elapsed() < AUTOSAVE_INTERVAL {
            return Ok(());
        }
        self.flush_draft(storage, false)
    }

    /// Writes the default draft if dirty, or when `force` is true.
    pub fn flush_draft(&mut self, storage: &Storage, force: bool) -> Result<(), String> {
        if !self.dirty && !force {
            return Ok(());
        }
        let text = self.editor.to_text();
        match storage.save_draft(&text) {
            Ok(()) => {
                self.dirty = false;
                self.dirty_since = None;
                if force {
                    self.save_state = SaveState::Saved;
                } else {
                    self.save_state = SaveState::AutoSaved;
                }
                Ok(())
            }
            Err(e) => {
                self.save_state = SaveState::SaveFailed(e.clone());
                Err(e)
            }
        }
    }

    /// Adjusts `scroll_row` (visual-row offset) so the cursor stays on screen.
    pub fn ensure_cursor_visible(&mut self, viewport_h: u16, viewport_w: u16) {
        let cursor_vrow = self.editor.cursor_visual_row(viewport_w);
        let v = viewport_h as usize;
        if cursor_vrow < self.scroll_row {
            self.scroll_row = cursor_vrow;
        } else if cursor_vrow >= self.scroll_row + v {
            self.scroll_row = cursor_vrow.saturating_sub(v.saturating_sub(1));
        }
    }
}
