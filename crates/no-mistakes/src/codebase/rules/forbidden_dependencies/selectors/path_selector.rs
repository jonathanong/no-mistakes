use crate::codebase::ts_resolver::normalize_path;
use globset::{Glob, GlobMatcher};
use std::path::{Path, PathBuf};

pub(in crate::codebase::rules::forbidden_dependencies) enum PathSelector {
    Literal(PathBuf),
    Pattern {
        matcher: GlobMatcher,
        relative: bool,
    },
}

impl PathSelector {
    pub(super) fn new(root: &Path, raw: &str) -> anyhow::Result<Self> {
        let path = normalize_path(&root.join(raw));
        Ok(match parse_glob(root, raw, &path)? {
            Some((glob, relative)) => Self::Pattern {
                matcher: glob.compile_matcher(),
                relative,
            },
            None => Self::Literal(path),
        })
    }

    pub(super) fn matches(&self, root: &Path, path: &Path) -> bool {
        match self {
            Self::Literal(prefix) => path.starts_with(prefix),
            Self::Pattern { matcher, relative } => {
                let candidate = if *relative {
                    path.strip_prefix(root).unwrap_or(path)
                } else {
                    path
                };
                matcher.is_match(candidate)
            }
        }
    }
}

pub(in crate::codebase::rules) fn validate(root: &Path, raw: &str) -> Result<(), globset::Error> {
    parse_glob(root, raw, &normalize_path(&root.join(raw))).map(drop)
}

fn parse_glob(root: &Path, raw: &str, path: &Path) -> Result<Option<(Glob, bool)>, globset::Error> {
    // Existing paths retain their literal meaning, even with glob syntax.
    if !raw.contains(['*', '?', '[', '{']) || path.is_file() || path.is_dir() {
        return Ok(None);
    }
    // Keep the invocation root out of glob syntax (it may itself contain brackets).
    let (pattern, relative) = match path.strip_prefix(root) {
        Ok(relative) => (relative, true),
        Err(_) => (path, false),
    };
    let pattern = pattern.to_string_lossy();
    #[cfg(windows)]
    let pattern = pattern.replace('\\', "/");
    Glob::new(&pattern).map(|glob| Some((glob, relative)))
}
