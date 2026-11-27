use super::lock::open_lock_file;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs::File;
use std::path::{Path, PathBuf};

pub(super) struct OpenedLock {
    pub(super) path: PathBuf,
    pub(super) file: File,
}

struct LockLocation {
    directory: PathBuf,
    temporary: bool,
}

impl LockLocation {
    fn user(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            temporary: false,
        }
    }

    fn private_temp(directory: PathBuf) -> Self {
        Self {
            directory,
            temporary: true,
        }
    }

    fn open(&self, path: &Path) -> Result<File> {
        if self.temporary {
            prepare_private_temp(&self.directory)?;
        } else {
            anyhow::ensure!(
                self.directory.is_absolute(),
                "lock directory must be absolute"
            );
            create_lock_directory(&self.directory)?;
        }
        open_lock_file(path)
    }
}

pub(super) fn open_current_lock() -> Option<OpenedLock> {
    if cfg!(test) || std::env::var_os("CARGO_BIN_EXE_no-mistakes").is_some() {
        return open_test_lock();
    }
    let mut locations = Vec::new();
    if let Some(directory) = std::env::var_os("NO_MISTAKES_LOCK_DIR").filter(|dir| !dir.is_empty())
    {
        locations.push(LockLocation::user(directory));
    }
    if let Some(project) = ProjectDirs::from("", "", "no-mistakes") {
        locations.extend(project.runtime_dir().map(LockLocation::user));
        locations.push(LockLocation::user(project.cache_dir()));
    }
    locations.extend(temporary_locations());
    open_first_location(locations)
}

fn open_first_location(locations: impl IntoIterator<Item = LockLocation>) -> Option<OpenedLock> {
    let mut failures = Vec::<FailedLocation>::new();
    for location in locations {
        let path = location.directory.join("invocation.lock");
        if failures.iter().any(|failed| failed.path == path) {
            continue;
        }
        match location.open(&path) {
            Ok(file) => {
                if !failures.is_empty() {
                    warn(&failures, Some(&path));
                }
                return Some(OpenedLock { path, file });
            }
            Err(error) => failures.push(FailedLocation { path, error }),
        }
    }
    warn(&failures, None);
    None
}

fn test_lock_path() -> PathBuf {
    std::env::temp_dir()
        .join("no-mistakes-test-locks")
        .join(format!("{}.lock", std::process::id()))
}

fn open_test_lock() -> Option<OpenedLock> {
    let path = test_lock_path();
    let result = create_lock_directory(path.parent().expect("test lock has a parent"))
        .and_then(|()| open_lock_file(&path));
    match result {
        Ok(file) => Some(OpenedLock { path, file }),
        Err(error) => {
            warn(&[FailedLocation { path, error }], None);
            None
        }
    }
}

pub(super) fn create_lock_directory(directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory).with_context(|| {
        format!(
            "creating no-mistakes invocation lock directory {}",
            directory.display()
        )
    })
}

struct FailedLocation {
    path: PathBuf,
    error: anyhow::Error,
}

fn warn(failures: &[FailedLocation], lock_file: Option<&Path>) {
    let failed: Vec<_> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "path": failure.path.to_string_lossy(),
                "error": format!("{:#}", failure.error),
            })
        })
        .collect();
    let warning = serde_json::json!({
        "level": "warning",
        "kind": "invocation_lock_fallback",
        "phase": "invocation.lock",
        "failed": failed,
        "lockFile": lock_file.map(Path::to_string_lossy),
    });
    crate::hooks::print_stderr(format_args!("{warning}"));
}

#[path = "lock_location/temp.rs"]
mod temp;
use temp::{prepare_private_temp, temporary_locations};

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "lock_location/tests.rs"]
mod tests;
