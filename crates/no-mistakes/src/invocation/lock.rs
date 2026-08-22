use super::{clock, InvocationError, InvocationErrorKind};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub(super) fn lock_path() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", "no-mistakes")
        .context("could not determine the current user's invocation lock directory")?;
    let directory = project_dirs
        .runtime_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project_dirs.cache_dir().to_path_buf());
    create_lock_directory(&directory)?;
    Ok(directory.join("invocation.lock"))
}

pub(super) fn create_lock_directory(directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory).with_context(|| {
        format!(
            "creating no-mistakes invocation lock directory {}",
            directory.display()
        )
    })
}

pub(super) fn acquire_lock(
    path: &Path,
    timeout: Option<Duration>,
    fail_on_lock: bool,
) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("opening invocation lock {}", path.display()))?;
    let started = clock::now();
    let mut first_attempt = true;
    let mut last_reported_secs = None;
    loop {
        if !first_attempt {
            if let Some(timeout) = timeout.filter(|timeout| started.elapsed() >= *timeout) {
                return Err(lock_timeout_error(timeout));
            }
        }
        match file.try_lock() {
            Ok(()) => {
                write_holder_pid(&file)?;
                return Ok(file);
            }
            Err(TryLockError::Error(error)) => return Err(lock_error(path, error)),
            Err(TryLockError::WouldBlock) if fail_on_lock => {
                return Err(InvocationError::new(
                    InvocationErrorKind::LockBusy,
                    "another no-mistakes invocation is already running",
                )
                .into());
            }
            Err(TryLockError::WouldBlock) => {
                report_lock_wait(path, started.elapsed(), &mut last_reported_secs);
            }
        }
        first_attempt = false;

        let sleep_for = match timeout {
            Some(timeout) => timeout
                .saturating_sub(started.elapsed())
                .min(LOCK_POLL_INTERVAL),
            None => LOCK_POLL_INTERVAL,
        };
        std::thread::sleep(sleep_for);
    }
}

fn write_holder_pid(file: &File) -> Result<()> {
    let mut file = file.try_clone()?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    write!(file, "{}", std::process::id())?;
    file.flush()?;
    Ok(())
}

fn report_lock_wait(path: &Path, elapsed: Duration, last_reported_secs: &mut Option<u64>) {
    let secs = elapsed.as_secs();
    if last_reported_secs.is_some_and(|previous| previous == secs) {
        return;
    }
    *last_reported_secs = Some(secs);
    let holder = read_holder_pid(path)
        .map(|pid| format!("pid {pid}"))
        .unwrap_or_else(|| "another no-mistakes invocation".to_string());
    eprintln!("waiting for lock held by {holder} for {secs}s");
}

fn read_holder_pid(path: &Path) -> Option<u32> {
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    contents.trim().parse().ok()
}

fn lock_timeout_error(timeout: Duration) -> anyhow::Error {
    InvocationError::new(
        InvocationErrorKind::LockTimeout,
        format!(
            "timed out after {} seconds waiting for another no-mistakes invocation",
            timeout.as_secs()
        ),
    )
    .into()
}

pub(super) fn lock_error(path: &Path, error: std::io::Error) -> anyhow::Error {
    anyhow::Error::new(error).context(format!("locking invocation file {}", path.display()))
}
