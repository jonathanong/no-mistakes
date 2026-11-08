mod render;
use super::InvocationObserver;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FilePass {
    Parse,
    ValueFlowExtract,
    ValueFlowFinalize,
    PostgresAnnotation,
}
impl FilePass {
    fn label(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::ValueFlowExtract => "value_flow.extract",
            Self::ValueFlowFinalize => "value_flow.finalize",
            Self::PostgresAnnotation => "postgres.annotation",
        }
    }
}
impl serde::Serialize for FilePass {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileDiagnostic {
    pass: FilePass,
    path: PathBuf,
    duration: Duration,
    units: u64,
}
#[derive(Default)]
pub(super) struct HotFiles {
    entries: [Vec<FileDiagnostic>; 4],
}
impl HotFiles {
    fn record(&mut self, entry: FileDiagnostic) {
        let entries = &mut self.entries[entry.pass as usize];
        if let Some(index) = entries.iter().position(|saved| saved.path == entry.path) {
            if (entries[index].duration, entries[index].units) >= (entry.duration, entry.units) {
                return;
            }
            entries.swap_remove(index);
        }
        entries.push(entry);
        entries.sort_unstable_by(|a, b| {
            b.duration
                .cmp(&a.duration)
                .then_with(|| a.path.cmp(&b.path))
                .then_with(|| b.units.cmp(&a.units))
        });
        entries.truncate(10);
    }
}
impl InvocationObserver {
    pub(crate) fn trace_file<T>(
        &self,
        pass: FilePass,
        path: &Path,
        units: u64,
        operation: impl FnOnce() -> T,
    ) -> T {
        self.trace_file_with_clock(pass, path, units, || self.snapshot_clock().0, operation)
    }
    fn trace_file_with_clock<T>(
        &self,
        pass: FilePass,
        path: &Path,
        units: u64,
        now: impl Fn() -> Instant,
        operation: impl FnOnce() -> T,
    ) -> T {
        let Some(files) = &self.hot_files else {
            return operation();
        };
        let started = now();
        let result = operation();
        let duration = now().duration_since(started);
        let entry = FileDiagnostic {
            pass,
            path: path.to_path_buf(),
            duration,
            units,
        };
        files
            .lock()
            .expect("hot file diagnostics mutex must not be poisoned")
            .record(entry);
        result
    }
    pub(super) fn hot_file_snapshot(&self) -> Vec<super::report::HotFileDiagnostic> {
        let Some(files) = &self.hot_files else {
            return Vec::new();
        };
        let entries = files.lock().unwrap().entries.clone();
        entries
            .into_iter()
            .flatten()
            .map(|entry| super::report::HotFileDiagnostic {
                pass: entry.pass,
                path: entry.path.to_string_lossy().into_owned(),
                duration_ms: entry.duration.as_secs_f64() * 1000.0,
                units: entry.units,
            })
            .collect()
    }
}
pub(crate) fn trace<T>(
    pass: FilePass,
    path: &Path,
    units: u64,
    operation: impl FnOnce() -> T,
) -> T {
    match super::current() {
        Some(observer)
            if pass == FilePass::Parse
                && observer.verbose()
                && FILE.with(|file| {
                    file.borrow().as_ref().is_some_and(|(owner, active)| {
                        *owner == std::sync::Arc::as_ptr(&observer) as usize && active == path
                    })
                }) =>
        {
            operation()
        }
        Some(observer) => observer.trace_file(pass, path, units, operation),
        None => operation(),
    }
}
thread_local! {
    static FILE: RefCell<Option<(usize, PathBuf)>> = const { RefCell::new(None) };
}
struct FileGuard(Option<(usize, PathBuf)>);
impl Drop for FileGuard {
    fn drop(&mut self) {
        FILE.with(|file| *file.borrow_mut() = self.0.take());
    }
}
pub(crate) fn with_file<T>(path: &Path, operation: impl FnOnce() -> T) -> T {
    let Some(observer) = super::current().filter(|observer| observer.verbose()) else {
        return operation();
    };
    let identity = std::sync::Arc::as_ptr(&observer) as usize;
    let _guard = FileGuard(FILE.with(|file| file.replace(Some((identity, path.to_path_buf())))));
    operation()
}
pub(crate) fn trace_current<T>(pass: FilePass, units: u64, operation: impl FnOnce() -> T) -> T {
    let Some(observer) = super::current().filter(|observer| observer.verbose()) else {
        return operation();
    };
    let identity = std::sync::Arc::as_ptr(&observer) as usize;
    let path = FILE.with(|file| {
        file.borrow()
            .as_ref()
            .filter(|(owner, _)| *owner == identity)
            .map(|(_, path)| path.clone())
    });
    match path {
        Some(path) => observer.trace_file(pass, &path, units, operation),
        None => operation(),
    }
}
#[cfg(test)]
mod fallback_tests;
#[cfg(test)]
mod tests;
