mod app;
mod editor;
mod input;
mod signals;
mod storage;
mod ui;

use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use app::App;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use storage::Storage;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!("tmptxt — terminal scratchpad with auto-save");
    println!();
    println!("Usage:");
    println!("  tmptxt           Open the scratchpad (default draft)");
    println!("  tmptxt --help    Show this help");
    println!("  tmptxt --version Show version");
}

fn print_version() {
    println!("tmptxt {VERSION}");
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                print_version();
                return Ok(());
            }
            other => {
                eprintln!("Unknown argument: {other}");
                eprintln!("Try `tmptxt --help`.");
                std::process::exit(2);
            }
        }
    }

    signals::init();

    let storage = match Storage::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tmptxt: {e}");
            std::process::exit(1);
        }
    };

    let draft_path = storage.draft_path.clone();
    let initial_text = match storage.load_draft() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tmptxt: failed to read draft: {e}");
            std::process::exit(1);
        }
    };

    enable_raw_mode()?;
    let mut stdout = stdout();
    // Do not enable mouse capture: tmptxt does not use the mouse, and capture prevents many
    // terminals from native click-drag selection / copy.
    execute!(stdout, EnterAlternateScreen, event::EnableBracketedPaste)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, storage, draft_path, initial_text);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        event::DisableBracketedPaste,
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("tmptxt: {e}");
        std::process::exit(1);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    storage: Storage,
    draft_path: PathBuf,
    initial_text: String,
) -> Result<(), String> {
    let mut app = App::new(draft_path, initial_text);
    let poll_timeout = Duration::from_millis(100);

    loop {
        if signals::take_interrupt() {
            app.flush_draft(&storage, true)?;
            break;
        }

        terminal
            .draw(|f| ui::render(f, &mut app))
            .map_err(|e| format!("draw failed: {e}"))?;

        if event::poll(poll_timeout).map_err(|e| format!("poll failed: {e}"))? {
            match event::read().map_err(|e| format!("read event failed: {e}"))? {
                event::Event::Key(key) => {
                    input::handle_key(&mut app, key, &storage)?;
                }
                event::Event::Paste(text) => {
                    if app.mode.is_editing() {
                        app.editor.insert_str(&text);
                        app.mark_dirty();
                    }
                }
                event::Event::Resize(_, _) => {}
                _ => {}
            }
        }

        app.tick_autosave(&storage)?;

        if app.should_quit {
            app.flush_draft(&storage, true)?;
            break;
        }
    }

    Ok(())
}
