<div align="center">

# tmptxt

**A tiny terminal scratchpad — auto-saves, one draft, zero fuss.**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-studentiz%2Ftmptxt-181717?logo=github)](https://github.com/studentiz/tmptxt)

</div>

> *Terminal scratchpad that auto-saves and resumes where you left off.*

---

## Table of contents

| | |
| :--- | :--- |
| [Why tmptxt](#why-tmptxt) | What it is (and is not) |
| [Features](#features) | What you get out of the box |
| [Installation](#installation) | macOS · Linux · Windows (full copy-paste flows) |
| [Uninstall](#uninstall) | Remove the binary and optional data |
| [Usage](#usage) | Day-to-day commands |
| [Keybindings](#keybindings) | Keyboard reference |
| [Data storage](#data-storage-location) | Where your draft lives |
| [Philosophy](#design-philosophy) | Why it stays small |
| [Non-goals](#non-goals) | What we deliberately skip |
| [License](#license) | Apache License 2.0 |
| [Publish to GitHub](#publish-to-github-for-maintainers) | One-time `gh` push |

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

**Goal:** from **any** folder in a terminal, type `tmptxt` and press Enter (same idea as `nano`).

Official source code: **[github.com/studentiz/tmptxt](https://github.com/studentiz/tmptxt)**.

Pick **one** section below — **macOS**, **Linux**, or **Windows** — and run the commands in order.

---

### All platforms — Rust toolchain (do this first)

1. Open **[rustup.rs](https://rustup.rs/)** and install Rust for your operating system.
2. When the installer finishes, **close every terminal window** and open a **new** one.
3. Check that Cargo works:

```bash
cargo --version
```

You should see a version number. If not, restart the computer and try again.

You also need **Git** to clone the repository:

| OS | If Git is missing |
| :--- | :--- |
| **macOS** | Install [Xcode Command Line Tools](https://developer.apple.com/download/all/) (`xcode-select --install`) or install Git from [git-scm.com](https://git-scm.com/download/mac). |
| **Linux** | e.g. Debian/Ubuntu: `sudo apt update && sudo apt install -y git build-essential` · Fedora: `sudo dnf install -y git gcc` |
| **Windows** | Install [Git for Windows](https://git-scm.com/download/win) and use **Git Bash** or **PowerShell** for the steps below. |

---

### Installation on macOS

Use **Terminal** (or iTerm2). The default shell is **zsh**.

**1.** Clone the repo and enter it (copy the whole block):

```bash
cd ~
git clone https://github.com/studentiz/tmptxt.git
cd tmptxt
```

**2.** Install the `tmptxt` binary into Cargo’s bin folder (`~/.cargo/bin`):

```bash
cargo install --path .
```

Wait until this finishes without errors.

**3.** **Close Terminal**, open a **new** window, then verify:

```bash
tmptxt --version
tmptxt
```

You do **not** need to `cd` into the project for these commands.

**4.** If `tmptxt` is **not found**, add Cargo’s bin directory to your PATH for zsh, then open a **new** terminal:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
tmptxt --version
```

---

### Installation on Linux

Use your usual terminal (**bash** is common). Commands below use `bash` for PATH fixes.

**1.** Clone the repo and enter it:

```bash
cd ~
git clone https://github.com/studentiz/tmptxt.git
cd tmptxt
```

**2.** Install:

```bash
cargo install --path .
```

**3.** **Close the terminal**, open a **new** one, then:

```bash
tmptxt --version
tmptxt
```

**4.** If `tmptxt` is **not found**, add `~/.cargo/bin` to PATH (bash), then open a **new** terminal:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
tmptxt --version
```

If you use **zsh** on Linux, replace `~/.bashrc` with `~/.zshrc` in the two lines above.

---

### Installation on Windows

Use **PowerShell** (recommended). Install Rust from [rustup.rs](https://rustup.rs/) for Windows, then install **Git** if you do not have `git` yet.

**1.** Clone and enter the repo (default clone location: your user folder):

```powershell
cd $env:USERPROFILE
git clone https://github.com/studentiz/tmptxt.git
cd tmptxt
```

**2.** Install:

```powershell
cargo install --path .
```

**3.** **Close PowerShell**, open a **new** window, then:

```powershell
tmptxt --version
tmptxt
```

**4.** If `tmptxt` is **not recognized**, add Cargo’s bin directory to your user PATH once, then **close and reopen** PowerShell:

```powershell
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$cargoBin*") {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$cargoBin", "User")
}
```

Open a **new** PowerShell window and run `tmptxt --version` again.

---

### Already cloned? (all platforms)

If you already have the project folder (for example you downloaded a ZIP), open a terminal **inside** that folder (where `Cargo.toml` is), then:

```bash
cargo install --path .
```

Then continue with the **verify** steps for your OS above (new terminal, `tmptxt --version`).

---

### Optional: copy the binary to `~/.local/bin` (macOS & Linux only)

Use this only if you prefer not to use `~/.cargo/bin` on your PATH.

```bash
mkdir -p ~/.local/bin
cd ~/tmptxt
cargo build --release
cp target/release/tmptxt ~/.local/bin/tmptxt
chmod +x ~/.local/bin/tmptxt
```

**macOS (zsh):**

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Linux (bash):**

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

Open a **new** terminal and run `tmptxt --version`.

---

## Uninstall

Uninstalling removes the **program** from Cargo’s bin directory (or your manual copy). Your **draft file is not removed** unless you run the optional data-deletion steps at the end.

### macOS or Linux — installed with `cargo install --path .`

In Terminal:

```bash
cargo uninstall tmptxt
```

Close all terminal windows.

### Windows — installed with `cargo install --path .`

In **PowerShell**:

```powershell
cargo uninstall tmptxt
```

Close all PowerShell windows.

### macOS or Linux — optional `~/.local/bin` copy

If you used the optional copy install:

```bash
rm -f ~/.local/bin/tmptxt
```

### Optional: delete saved notes (draft data)

**Linux:**

```bash
rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/tmptxt"
```

**macOS:**

```bash
rm -rf "$HOME/Library/Application Support/tmptxt"
```

**Windows (PowerShell):**

```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\tmptxt" -ErrorAction SilentlyContinue
```

### Future packaging

Homebrew, Scoop, or Linux distro packages may add their own install/remove commands later.

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

## Publish to GitHub (for maintainers)

The repository is intended to live at **[github.com/studentiz/tmptxt](https://github.com/studentiz/tmptxt)**. Creating it requires a one-time login with the [GitHub CLI](https://cli.github.com/).

From the project root, after installing `gh`:

```bash
gh auth login
./scripts/publish-to-github.sh
```

Or run the same steps by hand:

```bash
gh auth login
gh repo create tmptxt --public \
  --description "Minimal auto-saving terminal scratchpad (Rust)" \
  --source=. --remote=origin --push
```

If `origin` already exists and the empty repo is on GitHub:

```bash
git remote add origin https://github.com/studentiz/tmptxt.git   # skip if already added
git push -u origin main
```

---

## License

Licensed under the **Apache License, Version 2.0**. See [`LICENSE`](LICENSE) for the full text. A short attribution notice is in [`NOTICE`](NOTICE).
