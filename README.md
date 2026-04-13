<div align="center">

# 📝 tmptxt

**A tiny terminal scratchpad — auto-saves, one draft, zero fuss.**

<br>

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-studentiz%2Ftmptxt-181717?logo=github)](https://github.com/studentiz/tmptxt)

<br>

*Type `tmptxt` in any terminal, jot something down, quit.*
*Next time you type `tmptxt` — it's still there.*

</div>

<br>

## 💡 The idea

You're in a terminal. You need to **park a piece of text** — an API key, a command, a quick reminder — for a few minutes or a few days. You don't want to open an editor, pick a filename, or choose a folder.

**tmptxt** gives you a single scratchpad that is always one command away, saves itself, and stays out of your way.

- 🔑 Stash an API key like `OPENAI_API_KEY=sk-...` — paste it in, come back any time
- 🐳 Park a long `docker run` command you'll need again — no more clipboard accidents
- 🐛 Jot debug notes ("port 3001 works, 3000 doesn't") — beats scrolling through terminal history
- 🖥️ Remember SSH hosts, temp passwords, server IPs — one place, no file path to remember
- 📌 Leave yourself a config reminder — zero context-switch, zero extra apps

The common thread: **throwaway text you'd lose otherwise**, accessible from any directory by typing **one word**.

<br>

## ✨ What tmptxt does

- **📂 One command, anywhere** — type `tmptxt` from any folder to open the scratchpad
- **💾 Auto-saves everything** — saves in the background while you type, on exit (`Ctrl+X`), and even on `Ctrl+C`
- **📄 One draft, always** — a single `default.txt` in your OS data directory; no filenames, no folders to manage
- **📤 Export when needed** — `Ctrl+O` writes a copy to any path; the default draft stays untouched
- **🧹 Clear with confirmation** — `Ctrl+L` wipes the scratchpad, but only after you confirm (y/n)
- **🌍 Cross-platform** — macOS, Linux, Windows; full UTF-8 support (CJK friendly); adapts to terminal resize

<br>

## 🚀 Installation

After these steps, you can type `tmptxt` in **any** terminal window — the same way you'd type `nano` or `git`.

tmptxt is written in [Rust](https://www.rust-lang.org/). Your computer compiles it once during install. That requires two tools:

| Tool | What it does | How you get it |
| :--- | :--- | :--- |
| 🔧 **Git** | Downloads the tmptxt source code | Probably already installed — run `git --version` to check |
| 📦 **Cargo** | Compiles and installs tmptxt | Comes **free with Rust** — install Rust, get Cargo automatically |

> 💬 **Never heard of Cargo?** No worries. Cargo is just the build tool that ships with the Rust programming language. You don't download it separately — the Rust installer handles everything. The steps below walk you through it.

Pick **your operating system** and follow the steps in order. Every command is meant to be **copied and pasted as-is**.

<br>

---

### 🍎 macOS

> Your default terminal app is **Terminal** (Applications → Utilities). The default shell is **zsh**.

<br>

**Step 1 — Install Rust and Cargo**

Paste this into Terminal and press Enter:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

> 💬 This downloads the official Rust installer from [rustup.rs](https://rustup.rs/) and runs it with default settings. The `-y` flag means "accept defaults automatically".
>
> ⚠️ If your terminal says `curl: command not found`, run `xcode-select --install` first. That installs Apple's developer tools (which include `curl` and `git`). Wait for it to finish, then come back to Step 1.

Now tell the current terminal window where Cargo lives:

```bash
source "$HOME/.cargo/env"
```

> 💬 This loads Cargo into your current session. Future terminal windows will find it automatically.

Check that it worked — you should see a version number:

```bash
cargo --version
```

<br>

**Step 2 — Download tmptxt**

```bash
git clone https://github.com/studentiz/tmptxt.git ~/tmptxt
```

> 💬 This creates a folder called `tmptxt` inside your home directory (`~`), containing the source code.

<br>

**Step 3 — Build and install**

```bash
cd ~/tmptxt
cargo install --path .
```

> 💬 `cargo install --path .` compiles the project and places the `tmptxt` program into `~/.cargo/bin/` — a folder your system searches when you type a command.

<br>

**Step 4 — Verify**

**Close Terminal completely**, then open a **new** window. This ensures your PATH is up to date.

```bash
tmptxt --version
```

✅ You should see a version number like `tmptxt 0.1.0`.

<br>

**Step 5 — If `tmptxt` is not found** *(skip this if Step 4 worked)*

This means Cargo's folder (`~/.cargo/bin`) is not on your PATH yet. Fix it once:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

> 💬 This adds `~/.cargo/bin` to your shell's search path permanently. You only need to do this once. After running it, open a **new** Terminal window and try `tmptxt --version` again.

<br>

---

### 🐧 Linux

> Works on Ubuntu, Debian, Fedora, Arch, WSL, and most other distributions. Commands below assume **bash**.

<br>

**Step 1 — Install build dependencies**

Compiling tmptxt needs a C compiler, `curl`, and `git`. Pick your distro:

Ubuntu / Debian:

```bash
sudo apt-get update
sudo apt-get install -y curl build-essential git
```

Fedora:

```bash
sudo dnf install -y curl gcc gcc-c++ git
```

Arch:

```bash
sudo pacman -S --needed curl base-devel git
```

> 💬 These packages give Rust the tools it needs to compile programs. If they're already installed, the command does nothing — safe to run either way.

<br>

**Step 2 — Install Rust and Cargo**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

Load Cargo into the current session:

```bash
source "$HOME/.cargo/env"
```

Check:

```bash
cargo --version
```

<br>

**Step 3 — Download, build, install**

```bash
git clone https://github.com/studentiz/tmptxt.git ~/tmptxt
cd ~/tmptxt
cargo install --path .
```

<br>

**Step 4 — Verify**

**Close the terminal**, open a **new** one, then:

```bash
tmptxt --version
```

✅ You should see a version number.

<br>

**Step 5 — If `tmptxt` is not found** *(skip this if Step 4 worked)*

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

> 💬 If you use **zsh** instead of bash, replace `~/.bashrc` with `~/.zshrc` in both lines above.

Open a **new** terminal and try `tmptxt --version` again.

<br>

---

### 🪟 Windows

> Use **PowerShell** (pre-installed on Windows 10 and 11). You also need **Git** — if `git --version` fails, install [Git for Windows](https://git-scm.com/download/win) first.

<br>

**Step 1 — Install Rust and Cargo**

**Option A** — if `winget` works on your machine:

```powershell
winget install Rustlang.Rustup --accept-package-agreements --accept-source-agreements
```

**Option B** — no `winget`: open **[rustup.rs](https://rustup.rs/)** in a browser, download the installer, run it, and accept all defaults. If it asks to install **Visual Studio C++ Build Tools**, say yes.

> 💬 Rust needs a C/C++ compiler on Windows. The Visual Studio Build Tools provide that. This is a one-time setup — it won't affect your other programs.

⚠️ **After Rust finishes installing, close PowerShell and open a new window.** This is required so PowerShell can find the newly installed `cargo` command.

Check in the new window:

```powershell
cargo --version
```

<br>

**Step 2 — Download tmptxt**

```powershell
git clone https://github.com/studentiz/tmptxt.git $env:USERPROFILE\tmptxt
```

> 💬 This creates a `tmptxt` folder in your user directory (e.g. `C:\Users\YourName\tmptxt`).

<br>

**Step 3 — Build and install**

```powershell
cd $env:USERPROFILE\tmptxt
cargo install --path .
```

<br>

**Step 4 — Verify**

**Close PowerShell**, open a **new** window, then:

```powershell
tmptxt --version
```

✅ You should see a version number.

<br>

**Step 5 — If `tmptxt` is not recognized** *(skip this if Step 4 worked)*

```powershell
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$cargoBin*") {
  [Environment]::SetEnvironmentVariable("Path", "$currentPath;$cargoBin", "User")
}
```

> 💬 This adds Cargo's bin directory to your user PATH permanently. You only need to do this once.

Close PowerShell, open a new window, and try `tmptxt --version` again.

<br>

---

## 🎯 Usage

```bash
tmptxt            # open the scratchpad
tmptxt --help     # show help
tmptxt --version  # show version
```

tmptxt always opens the **same default draft**. It does not accept a filename argument — that's by design. One draft, always there, always one command away.

<br>

## ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| `Ctrl+X` | 💾 Save and exit |
| `Ctrl+O` | 📤 Save As — export a copy to a path you type (`Esc` cancels) |
| `Ctrl+L` | 🧹 Clear the scratchpad (asks y/n first) |
| `Arrow keys` | Move cursor |
| `Home` / `End` | Jump to start / end of line |
| `PageUp` / `PageDown` | Scroll by one screen |
| `Enter` | New line |
| `Backspace` / `Delete` | Delete character / join lines |

<br>

## 📁 Data storage

Your draft lives in the **OS user data directory** — never next to the installed program:

| OS | File path |
| :--- | :--- |
| 🐧 Linux | `~/.local/share/tmptxt/tmptxt/default.txt` |
| 🍎 macOS | `~/Library/Application Support/tmptxt/tmptxt/default.txt` |
| 🪟 Windows | `%LOCALAPPDATA%\tmptxt\tmptxt\default.txt` |

**Save As** (`Ctrl+O`) exports a copy to any path you choose; the app continues to use `default.txt`.

<br>

## 🗑️ Uninstall

Remove the program (all platforms):

```bash
cargo uninstall tmptxt
```

Optionally, delete the source code folder:

```bash
rm -rf ~/tmptxt
```

Optionally, delete your saved draft data:

| OS | Command |
| :--- | :--- |
| 🐧 Linux | `rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/tmptxt"` |
| 🍎 macOS | `rm -rf "$HOME/Library/Application Support/tmptxt"` |
| 🪟 Windows | `Remove-Item -Recurse -Force "$env:LOCALAPPDATA\tmptxt"` |

<br>

## 🧭 Design philosophy

- **💾 Auto-save** — no manual save, no "unsaved changes" prompts
- **📄 One draft** — no filenames, no folders, no decisions
- **🎯 Minimal** — does one thing well, then gets out of the way

<br>

## 🚫 Non-goals

tmptxt is not trying to become a text editor. It will not grow into multi-file management, tabs, syntax highlighting, search/replace, plugins, cloud sync, or accounts. If you need those, use a dedicated editor — keep tmptxt for the text that doesn't deserve a file.

<br>

---

<details>
<summary>🔧 <strong>For maintainers: publish to GitHub</strong></summary>

```bash
gh auth login
gh repo create tmptxt --public \
  --description "Minimal auto-saving terminal scratchpad (Rust)" \
  --source=. --remote=origin --push
```

Or push to an existing remote:

```bash
git remote add origin https://github.com/studentiz/tmptxt.git
git push -u origin main
```

</details>

<br>

## 📜 License

[Apache License 2.0](LICENSE) — see [`NOTICE`](NOTICE) for attribution.
