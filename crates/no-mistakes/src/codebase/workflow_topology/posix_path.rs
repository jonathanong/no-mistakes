//! Minimal POSIX-style path helpers (`path.posix.normalize` /
//! `path.posix.dirname` analogs from Node's `path` module), sufficient for
//! the paths this module handles: always relative, repo-relative, forward-
//! slash workflow/job paths — never absolute, never containing a drive
//! letter or a trailing-slash edge case that would need a fuller
//! implementation.

/// Collapses `.` segments and resolves `..` against preceding segments.
pub fn normalize(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => continue,
            ".." => match segments.last() {
                Some(&last) if last != ".." => {
                    segments.pop();
                }
                _ => segments.push(".."),
            },
            other => segments.push(other),
        }
    }
    if segments.is_empty() {
        ".".to_string()
    } else {
        segments.join("/")
    }
}

/// The parent directory of `path`, or `"."` when `path` has no `/`.
pub fn dirname(path: &str) -> String {
    match path.rfind('/') {
        Some(0) => "/".to_string(),
        Some(index) => path[..index].to_string(),
        None => ".".to_string(),
    }
}
