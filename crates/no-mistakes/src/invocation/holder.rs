//! Holder metadata that the lock owner records in the lock file.
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{ffi::OsString, path::Path};

const MAX_COMMAND_CHARS: usize = 512;

/// Who holds the machine-wide invocation lock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockHolder {
    pub pid: u32,
    /// Working directory of the holder; a pid alone is ambiguous across sandboxes.
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    /// Executable basename only for new records; arguments are never recorded.
    pub command: String,
    /// Unix epoch milliseconds when the holder acquired the lock; 0 when unknown.
    #[serde(default)]
    pub started_at_ms: u64,
    /// Whole seconds the holder has held the lock, computed when read.
    #[serde(default, skip_deserializing)]
    pub age_seconds: Option<u64>,
}

pub(super) fn sidecar_path(lock: &Path) -> std::path::PathBuf {
    let mut name = lock.as_os_str().to_owned();
    name.push(".holder");
    name.into()
}

pub(super) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

impl LockHolder {
    pub(super) fn current() -> Self {
        Self {
            pid: std::process::id(),
            cwd: std::env::current_dir()
                .ok()
                .map(|cwd| cwd.to_string_lossy().into_owned()),
            command: command_name(std::env::args_os()),
            started_at_ms: now_ms(),
            age_seconds: None,
        }
    }

    /// Parse lock file contents: the JSON record, or a legacy bare pid.
    pub(super) fn parse(contents: &str) -> Option<Self> {
        let contents = contents.trim();
        if let Ok(holder) = serde_json::from_str::<Self>(contents) {
            return Some(holder);
        }
        contents.parse().ok().map(|pid| Self {
            pid,
            cwd: None,
            command: String::new(),
            started_at_ms: 0,
            age_seconds: None,
        })
    }

    pub(super) fn with_age(mut self) -> Self {
        if self.started_at_ms > 0 {
            self.age_seconds = Some(now_ms().saturating_sub(self.started_at_ms) / 1000);
        }
        self
    }

    /// Read best-effort metadata, which can name a previous holder during handoff.
    /// A separate file remains readable on platforms with mandatory file locks.
    pub(super) fn read(lock: &Path) -> Option<Self> {
        let contents = std::fs::read_to_string(sidecar_path(lock)).ok()?;
        Self::parse(&contents).map(Self::with_age)
    }

    /// Record this holder after the lock is acquired, replacing any previous record.
    pub(super) fn record(&self, lock: &Path) -> std::io::Result<()> {
        let sidecar = sidecar_path(lock);
        // NamedTempFile creates a unique file (0600 on Unix), so a
        // leftover staging file cannot broaden permissions or expose arguments.
        let mut staged = tempfile::NamedTempFile::new_in(lock.parent().unwrap_or(Path::new(".")))?;
        serde_json::to_writer(staged.as_file_mut(), self)?;
        staged.persist(sidecar).map_err(|error| error.error)?;
        Ok(())
    }

    /// Human description used in lock messages.
    pub fn describe(&self) -> String {
        let mut details = Vec::new();
        if let Some(cwd) = self.cwd.as_deref().filter(|cwd| !cwd.is_empty()) {
            details.push(format!("cwd {cwd}"));
        }
        if !self.command.is_empty() {
            details.push(format!("command `{}`", self.command));
        }
        if let Some(age) = self.age_seconds {
            details.push(format!("held for {age}s"));
        }
        match details.is_empty() {
            true => format!("pid {}", self.pid),
            false => format!("pid {} ({})", self.pid, details.join(", ")),
        }
    }
}

fn command_name(args: impl IntoIterator<Item = OsString>) -> String {
    args.into_iter()
        .next()
        .and_then(|program| {
            Path::new(&program).file_name().map(|name| {
                name.to_string_lossy()
                    .chars()
                    .take(MAX_COMMAND_CHARS)
                    .collect()
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "holder/tests.rs"]
mod tests;
