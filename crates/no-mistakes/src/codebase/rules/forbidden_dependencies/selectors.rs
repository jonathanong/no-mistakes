use super::{helpers, Options, RuleFinding, RULE_ID};
use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use crate::codebase::ts_resolver::normalize_path;
use globset::{Glob, GlobMatcher, GlobSet};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

enum PathSelector {
    Literal(PathBuf),
    Pattern {
        matcher: GlobMatcher,
        relative: bool,
    },
}

impl PathSelector {
    fn new(root: &Path, raw: &str) -> anyhow::Result<Self> {
        let path = normalize_path(&root.join(raw));
        // Existing files and directories retain their literal meaning, even with glob syntax.
        if path.is_file() || path.is_dir() || !raw.contains(['*', '?', '[', '{']) {
            return Ok(Self::Literal(path));
        }
        // Keep the invocation root out of glob syntax (it may itself contain brackets).
        let (pattern, relative) = match path.strip_prefix(root) {
            Ok(relative) => (relative, true),
            Err(_) => (path.as_path(), false),
        };
        let pattern = pattern.to_string_lossy();
        #[cfg(windows)]
        let pattern = pattern.replace('\\', "/");
        Ok(Self::Pattern {
            matcher: Glob::new(&pattern)?.compile_matcher(),
            relative,
        })
    }

    fn matches(&self, root: &Path, path: &Path) -> bool {
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

struct Exception {
    roots: Vec<PathSelector>,
    allowed: Option<GlobSet>,
}

pub(super) struct Selectors<'a> {
    root: &'a Path,
    roots: Vec<(&'a str, PathSelector)>,
    excluded: Vec<PathSelector>,
    exceptions: Vec<Exception>,
}

impl<'a> Selectors<'a> {
    pub(super) fn new(
        root: &'a Path,
        anchor: &ConfigAnchor,
        opts: &'a Options,
    ) -> Result<Self, Box<RuleFinding>> {
        let error = |key: &str, index: usize, error: anyhow::Error| {
            Box::new(anchor.emit(
                RULE_ID,
                anchor.option_item_line(key, index),
                format!("{RULE_ID}: invalid glob pattern in {key}: {error}"),
                None,
                None,
            ))
        };
        let roots = opts
            .roots
            .iter()
            .enumerate()
            .map(|(i, raw)| {
                PathSelector::new(root, raw)
                    .map(|selector| (raw.as_str(), selector))
                    .map_err(|e| error("roots", i, e))
            })
            .collect::<Result<_, _>>()?;
        let excluded = opts
            .exclude_roots
            .iter()
            .enumerate()
            .map(|(i, raw)| PathSelector::new(root, raw).map_err(|e| error("excludeRoots", i, e)))
            .collect::<Result<_, _>>()?;
        let mut exceptions = Vec::new();
        for (index, exception) in opts.exceptions.iter().enumerate() {
            let nested_error = |key: &str, item: usize, error: anyhow::Error| {
                Box::new(anchor.emit(
                    RULE_ID,
                    anchor.nested_option_item_line("exceptions", index, key, item),
                    format!(
                        "{RULE_ID}: invalid glob pattern in exceptions[{index}].{key}: {error}"
                    ),
                    None,
                    None,
                ))
            };
            let roots = exception
                .roots
                .iter()
                .enumerate()
                .map(|(i, raw)| {
                    PathSelector::new(root, raw).map_err(|e| nested_error("roots", i, e))
                })
                .collect::<Result<_, _>>()?;
            let allowed = helpers::build_globset(&exception.allowed_files).map_err(|e| {
                let item = exception
                    .allowed_files
                    .iter()
                    .position(|raw| Glob::new(raw).is_err())
                    .unwrap_or_default();
                nested_error("allowedFiles", item, e)
            })?;
            exceptions.push(Exception { roots, allowed });
        }
        Ok(Self {
            root,
            roots,
            excluded,
            exceptions,
        })
    }

    pub(super) fn select(
        &self,
        graph: &DepGraph,
        universe: Option<&crate::fx::PathSet>,
    ) -> BTreeMap<PathBuf, &'a str> {
        graph
            .all_files()
            .filter_map(NodeId::as_file)
            .filter_map(|path| {
                if universe.is_some_and(|files| !files.contains(path))
                    || self
                        .excluded
                        .iter()
                        .any(|selector| selector.matches(self.root, path))
                {
                    return None;
                }
                self.roots
                    .iter()
                    .find(|(_, selector)| selector.matches(self.root, path))
                    .map(|(raw, _)| (path.to_path_buf(), *raw))
            })
            .collect()
    }

    pub(super) fn allows(&self, source: &Path, target: &Path) -> bool {
        let relative = target
            .strip_prefix(self.root)
            .unwrap_or(target)
            .to_string_lossy()
            .replace('\\', "/");
        self.exceptions.iter().any(|exception| {
            exception
                .roots
                .iter()
                .any(|selector| selector.matches(self.root, source))
                && exception
                    .allowed
                    .as_ref()
                    .is_some_and(|allowed| allowed.is_match(&relative))
        })
    }
}
