<div align="center">

# tmptxt

**A tiny terminal scratchpad — auto-saves, one draft, zero fuss.**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

</div>

> *Terminal scratchpad that auto-saves and resumes where you left off.*

---

## Table of contents

| | |
| :--- | :--- |
| [Why tmptxt](#why-tmptxt) | What it is (and is not) |
| [Features](#features) | What you get out of the box |
| [Installation](#installation) | Copy-paste friendly, macOS / Linux / Windows |
| [Uninstall](#uninstall) | Remove the binary and optional data |
| [Usage](#usage) | Day-to-day commands |
| [Keybindings](#keybindings) | Keyboard reference |
| [Data storage](#data-storage-location) | Where your draft lives |
| [Philosophy](#design-philosophy) | Why it stays small |
| [Non-goals](#non-goals) | What we deliberately skip |
| [License](#license) | Apache License 2.0 |

---

## Why tmptxt

tmptxt is a **low-friction scratch surface** in the terminal: paste snippets, jot reminders, and let it persist for you. It is **not** a full editor, IDE, notes vault, or task manager — just a single auto-saving draft, like digital scrap paper.

---

## Features

| | |
| :--- | :--- |
| **Editing** | Multi-line buffer, UTF-8 (including CJK), bracketed paste, resize-aware layout |
| **Persistence** | One default draft (`default.txt`); dirty flag + timed auto-save; save on exit (`Ctrl+X`) and best-effort on interrupt |
| **Export** | **Save As** writes a copy elsewhere; the app always reopens the same default draft |
| **Safety** | **Clear** asks for confirmation before wiping the scratchpad |
| **UI** | Full-screen TUI: header, editor, shortcut bar — minimal and readable |

---

## Installation

**Goal:** open **any** terminal, type `tmptxt`, press Enter — same idea as `nano` once it is on your **PATH**.

Follow **either** the **macOS / Linux** path **or** the **Windows** path below.

### 0. One-time: install Rust (skip if `cargo` already works)

1. Open [rustup.rs](https://rustup.rs/) and install Rust for your OS.
2. When the installer finishes, **close every terminal window** and open a **new** one.
3. Copy-paste:

```bash
cargo --version
```

If you see a version number, continue. If not, restart the computer and try step 3 again.

### 1. Open the tmptxt project folder in a terminal

The folder must contain **`Cargo.toml`**.

**macOS / Linux** — edit the path, then paste:

```bash
cd ~/path/to/tmptxt
```

Examples:

```bash
cd ~/Desktop/tmptxt
```

```bash
cd ~/Downloads/tmptxt
```

**Git clone** (replace the URL):

```bash
cd ~
git clone <repository-url> tmptxt
cd tmptxt
```

**Windows (PowerShell)** — edit the path:

```powershell
cd $env:USERPROFILE\Desktop\tmptxt
```

### 2a. Install on macOS or Linux

Paste, wait until it finishes with no errors:

```bash
cargo install --path .
```

Then **close this terminal**, open a **new** one, and run:

```bash
tmptxt --version
```

| Result | What to do |
| :--- | :--- |
| Version prints | Done — go to [§3](#3-check-that-it-works-like-nano). |
| `command not found` | Paste **one** block below, open a **new** terminal, run `tmptxt --version` again. |

**macOS (zsh — default; try this first):**

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Linux (bash):**

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 2b. Install on Windows

In **PowerShell**, from the tmptxt folder ([§1](#1-open-the-tmptxt-project-folder-in-a-terminal)):

```powershell
cargo install --path .
```

**Close PowerShell**, open a **new** window, then:

```powershell
tmptxt --version
```

| Result | What to do |
| :--- | :--- |
| Version prints | Go to [§3](#3-check-that-it-works-like-nano). |
| Command not found | Paste **once**, press Enter, **close and reopen** PowerShell, then `tmptxt --version` again. |

```powershell
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$cargoBin*") {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$cargoBin", "User")
}
```

### 3. Check that it works (like nano)

Use a **new** terminal. You should **not** need `cd` into the project.

**macOS / Linux:**

```bash
tmptxt --version
tmptxt
```

**Windows:**

```powershell
tmptxt --version
tmptxt
```

### 4. Optional fallback (macOS / Linux only)

If `cargo install --path .` failed or you prefer a manual copy:

```bash
mkdir -p ~/.local/bin
cargo build --release
cp target/release/tmptxt ~/.local/bin/tmptxt
chmod +x ~/.local/bin/tmptxt
```

**zsh:**

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**bash:** use `~/.bashrc` instead of `~/.zshrc` in both lines.

Open a **new** terminal and run `tmptxt --version`.

---

## Uninstall

### Installed with `cargo install --path .`

**macOS, Linux, or Windows** (Terminal or PowerShell):

```bash
cargo uninstall tmptxt
```

Close all terminals. This removes the binary from Cargo’s install folder; it **does not** delete your draft.

### Optional `~/.local/bin` copy (macOS / Linux)

```bash
rm -f ~/.local/bin/tmptxt
```

### Delete saved notes (optional)

| OS | Command |
| :--- | :--- |
| **Linux** | `rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/tmptxt"` |
| **macOS** | `rm -rf "$HOME/Library/Application Support/tmptxt"` |
| **Windows** | `Remove-Item -Recurse -Force "$env:LOCALAPPDATA\tmptxt" -ErrorAction SilentlyContinue` |

**Linux** (copy-paste):

```bash
rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/tmptxt"
```

**macOS** (copy-paste):

```bash
rm -rf "$HOME/Library/Application Support/tmptxt"
```

**Windows (PowerShell)** (copy-paste):

```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\tmptxt" -ErrorAction SilentlyContinue
```

### Future packaging

Homebrew, Scoop, or distro packages may add their own flows; the steps above work without them.

---

## Usage

```bash
tmptxt
```

```bash
tmptxt --help
tmptxt --version
```

tmptxt does **not** open arbitrary files by path; it always opens the **default draft**.

---

## Keybindings

| Binding | Action |
| :--- | :--- |
| `Ctrl+X` | Save draft and exit |
| `Ctrl+O` | Save As (path input; `Esc` cancels) |
| `Ctrl+L` | Clear scratchpad (**y** / **n** / `Esc` confirm) |
| Arrows, `Home`, `End` | Move cursor |
| `PageUp` / `PageDown` | Scroll by about one screen |
| `Enter` | New line |
| `Backspace` / `Delete` | Delete / join lines at edges |

---

## Data storage location

Draft data lives under the **OS user data directory**, never beside the binary:

| OS | Path |
| :--- | :--- |
| **Linux** | `$XDG_DATA_HOME/tmptxt/tmptxt` or `~/.local/share/tmptxt/tmptxt` |
| **macOS** | `~/Library/Application Support/tmptxt/tmptxt` |
| **Windows** | `%LOCALAPPDATA%\tmptxt\tmptxt\` |

Default file: **`default.txt`**. **Save As** only exports a copy; the app keeps using the default draft.

---

## Design philosophy

- **Auto-save** over manual save rituals for scratch content
- **One draft** to keep mental overhead low
- **Small surface** over feature creep

---

## Non-goals

tmptxt is **not** aiming to replace a real editor. There is no multi-file UI, tabs, syntax highlighting, Markdown preview, search/replace, sync, accounts, or task features. Use a proper editor when you need those — keep tmptxt for quick throwaway text.

---

## License

Licensed under the **Apache License, Version 2.0**. See [`LICENSE`](LICENSE) for the full text. A short attribution notice is in [`NOTICE`](NOTICE).
