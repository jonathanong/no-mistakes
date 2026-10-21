//! Opt-in locations for long-running annotation evaluation.
use std::path::Path;

pub(super) fn trace<T>(
    label: &'static str,
    path: &Path,
    offset: Option<u32>,
    operation: impl FnOnce() -> T,
) -> T {
    located(label, path, offset, operation)
}

fn located<T>(
    label: &'static str,
    path: &Path,
    location: Option<u32>,
    operation: impl FnOnce() -> T,
) -> T {
    let Some(observer) = crate::diagnostics::current().filter(|observer| observer.detailed())
    else {
        return operation();
    };
    observer.increment(label, 1);
    let result = if observer.progress_enabled() {
        let location = match location {
            Some(offset) => format!("{label} {}:{offset}", path.display()),
            None => format!("{label} {}", path.display()),
        };
        observer.activity(
            &location,
            crate::diagnostics::TimingKind::Parallel,
            operation,
        )
    } else {
        operation()
    };
    observer.increment("postgres.annotation.completed_activities", 1);
    match label {
        "postgres.annotation.root" => observer.increment("postgres.annotation.roots_completed", 1),
        "postgres.annotation.module" => {
            observer.increment("postgres.annotation.module_initializations_completed", 1)
        }
        _ => {}
    }
    result
}

#[cfg(test)]
mod tests;
