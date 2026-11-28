//! Cross-platform LCOV source-file path normalization.

mod clean;
mod helpers;

use clean::{clean_path, is_foreign_absolute, strip_prefix_match, PathKind};
use helpers::{strip_ci_work_segments, strip_file_url};
use std::fmt;

/// Configuration for normalizing LCOV `SF:` source file paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizeOptions {
    pub root: Option<String>,
    pub strip_prefixes: Vec<String>,
    pub strip_ci_work_prefix: bool,
    pub reject_outside_root: bool,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        Self {
            root: None,
            strip_prefixes: Vec::new(),
            strip_ci_work_prefix: true,
            reject_outside_root: false,
        }
    }
}

impl NormalizeOptions {
    pub fn with_root(mut self, root: impl Into<String>) -> Self {
        self.root = Some(root.into());
        self
    }

    pub fn with_strip_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.strip_prefixes.push(prefix.into());
        self
    }

    pub fn with_strip_ci_work_prefix(mut self, enabled: bool) -> Self {
        self.strip_ci_work_prefix = enabled;
        self
    }

    pub fn with_reject_outside_root(mut self, enabled: bool) -> Self {
        self.reject_outside_root = enabled;
        self
    }
}

/// Errors returned when an LCOV path fails normalization or containment checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathNormalizeError {
    Empty,
    ContainsNul,
    EscapesRoot,
    ForeignAbsolutePath,
    OutsideRoot,
}

impl fmt::Display for PathNormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "path is empty after normalization",
            Self::ContainsNul => "path contains a NUL byte",
            Self::EscapesRoot => "relative path escapes root via '..'",
            Self::ForeignAbsolutePath => "path uses a foreign OS absolute root",
            Self::OutsideRoot => "absolute path is outside the configured root",
        };
        f.write_str(message)
    }
}

impl std::error::Error for PathNormalizeError {}

/// Normalizes an LCOV `SF:` path relative to `options.root` / `strip_prefixes`.
pub fn normalize_path(raw: &str, options: &NormalizeOptions) -> Result<String, PathNormalizeError> {
    if raw.contains('\0') {
        return Err(PathNormalizeError::ContainsNul);
    }
    if raw.is_empty() {
        return Err(PathNormalizeError::Empty);
    }

    let unurl = strip_file_url(raw)?;
    let cleaned = clean_path(&unurl)?;

    for raw_prefix in &options.strip_prefixes {
        if let Ok(prefix) = clean_path(raw_prefix) {
            if let Some(res) = strip_prefix_match(&cleaned, &prefix) {
                return res;
            }
        }
    }

    let mut foreign = false;
    if let Some(raw_root) = options.root.as_deref() {
        if let Ok(root) = clean_path(raw_root) {
            if let Some(res) = strip_prefix_match(&cleaned, &root) {
                return res;
            }
            foreign = is_foreign_absolute(cleaned.kind, root.kind);
        }
    }

    if cleaned.kind == PathKind::Relative {
        return if cleaned.text.is_empty() {
            Err(PathNormalizeError::Empty)
        } else {
            Ok(cleaned.text)
        };
    }

    if options.strip_ci_work_prefix
        && matches!(cleaned.kind, PathKind::PosixAbs | PathKind::WindowsDrive(_))
    {
        if let Some(stripped) = strip_ci_work_segments(cleaned.suffix_after_drive()) {
            return Ok(stripped);
        }
    }

    if options.reject_outside_root {
        return Err(if foreign {
            PathNormalizeError::ForeignAbsolutePath
        } else {
            PathNormalizeError::OutsideRoot
        });
    }

    Ok(cleaned.text)
}
