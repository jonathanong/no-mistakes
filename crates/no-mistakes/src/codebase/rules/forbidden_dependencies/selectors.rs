use super::{helpers, Options, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use globset::{Glob, GlobSet};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod path_selector;
pub(in crate::codebase::rules::forbidden_dependencies) use path_selector::validate;
use path_selector::PathSelector;

mod audit;

struct Exception {
    roots: Vec<PathSelector>,
    allowed: Option<GlobSet>,
    usage: crate::codebase::rules::allow_audit::AllowUsage,
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
            exceptions.push(Exception {
                roots,
                allowed,
                usage: crate::codebase::rules::allow_audit::AllowUsage::new(
                    exception.allowed_files.len(),
                ),
            });
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
        let mut suppressed = false;
        for exception in &self.exceptions {
            let source_matches = exception
                .roots
                .iter()
                .any(|selector| selector.matches(self.root, source));
            let matches = exception
                .allowed
                .as_ref()
                .map(|allowed| allowed.matches(&relative))
                .unwrap_or_default();
            suppressed |= exception.usage.suppress(
                (0..exception.usage_count())
                    .map(|index| source_matches && matches.contains(&index)),
            );
        }
        suppressed
    }
}

impl Exception {
    fn usage_count(&self) -> usize {
        self.allowed.as_ref().map_or(0, GlobSet::len)
    }
}
