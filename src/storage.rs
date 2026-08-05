//! User-data directory draft persistence (never uses the install directory).

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

pub struct Storage {
    pub draft_path: PathBuf,
}

/// Restricts `path` to the current user (dir: 700, file: 600).  On non-Unix
/// platforms there is no POSIX mode, so this is a no-op (Windows uses ACLs).
#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}
#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

impl Storage {
    /// Resolves `tmptxt` data dir and ensures it exists (owner-only).
    pub fn new() -> Result<Self, String> {
        let dirs = ProjectDirs::from("org", "tmptxt", "tmptxt")
            .ok_or_else(|| "could not resolve user data directory".to_string())?;
        let data_dir = dirs.data_dir().to_path_buf();
        fs::create_dir_all(&data_dir).map_err(|e| {
            format!(
                "could not create data directory {}: {e}",
                data_dir.display()
            )
        })?;
        // The draft may hold secrets; keep the data dir and any existing draft
        // readable only by the current user on Unix (700 / 600).
        set_mode(&data_dir, 0o700).map_err(|e| {
            format!(
                "could not restrict data directory {}: {e}",
                data_dir.display()
            )
        })?;
        let draft_path = data_dir.join("default.txt");
        if draft_path.exists() {
            set_mode(&draft_path, 0o600).map_err(|e| {
                format!("could not restrict draft {}: {e}", draft_path.display())
            })?;
        }
        Ok(Self { draft_path })
    }

    /// Loads the default draft. Missing file yields empty content.
    pub fn load_draft(&self) -> Result<String, String> {
        if !self.draft_path.exists() {
            return Ok(String::new());
        }
        let mut f = fs::File::open(&self.draft_path)
            .map_err(|e| format!("open {}: {e}", self.draft_path.display()))?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .map_err(|e| format!("read {}: {e}", self.draft_path.display()))?;
        Ok(buf)
    }

    /// Atomically writes the default draft (best-effort crash safety).
    pub fn save_draft(&self, content: &str) -> Result<(), String> {
        self.atomic_write(&self.draft_path, content, true)
    }

    /// Exports content to a user-chosen path (does not change the default draft file).
    pub fn save_as(&self, path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(format!("directory does not exist: {}", parent.display()));
            }
        }
        self.atomic_write(path, content, false)
    }

    fn atomic_write(&self, path: &Path, content: &str, private: bool) -> Result<(), String> {
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
        if let Some(p) = parent {
            fs::create_dir_all(p).map_err(|e| format!("create {}: {e}", p.display()))?;
        }

        let tmp = path.with_extension("tmp");
        {
            let mut opts = fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true);
            // Draft temp file starts owner-only so content never sits world-readable
            // even between create and the final rename.
            #[cfg(unix)]
            if private {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }
            let mut f = opts.open(&tmp).map_err(|e| {
                format!(
                    "could not write temporary file {}: {e}",
                    tmp.display()
                )
            })?;
            f.write_all(content.as_bytes())
                .map_err(|e| format!("write {}: {e}", tmp.display()))?;
            f.sync_all()
                .map_err(|e| format!("sync {}: {e}", tmp.display()))?;
        }

        if path.exists() {
            fs::remove_file(path).map_err(|e| {
                let _ = fs::remove_file(&tmp);
                format!("could not replace {}: {e}", path.display())
            })?;
        }

        fs::rename(&tmp, path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("could not finalize save to {}: {e}", path.display())
        })?;

        if private {
            set_mode(path, 0o600).map_err(|e| {
                format!("could not restrict {}: {e}", path.display())
            })?;
        }

        Ok(())
    }
}
