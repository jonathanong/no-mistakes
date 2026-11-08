use super::super::path_filter::RulePathFilter;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::{schema::RuleDef, DisableDirectivePolicy, NoMistakesConfig};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct MissingFiles {
    root: PathBuf,
    inventory: Arc<FileInventory>,
    filter: RulePathFilter,
    roots: Vec<PathBuf>,
    skip: crate::fx::FxHashSet<String>,
}

impl MissingFiles {
    pub(crate) fn new(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        sources: Option<&SourceStore>,
    ) -> Option<Self> {
        if config.missing_allow_audit_scope_incomplete
            || config.report_unused_allow_entries == DisableDirectivePolicy::Ignore
        {
            return None;
        }
        let sources = sources.filter(|sources| sources.inventory_is_complete())?;
        let mut inferred = sources.inferred_roots_for_projects(
            root,
            rule.projects
                .iter()
                .filter_map(|name| config.projects.get(name)),
        );
        Self::from_inferred(root, config, rule, sources, &mut inferred)
    }

    pub(crate) fn new_with_inferred(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        sources: Option<&SourceStore>,
        inferred: &crate::codebase::config::InferredRoots,
    ) -> Option<Self> {
        if config.missing_allow_audit_scope_incomplete
            || config.report_unused_allow_entries == DisableDirectivePolicy::Ignore
        {
            return None;
        }
        let sources = sources.filter(|sources| sources.inventory_is_complete())?;
        Self::from_inferred(root, config, rule, sources, &mut inferred.clone())
    }

    fn from_inferred(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        sources: &SourceStore,
        inferred: &mut crate::codebase::config::InferredRoots,
    ) -> Option<Self> {
        Some(Self {
            root: crate::codebase::ts_resolver::normalize_path(root),
            inventory: Arc::clone(sources.inventory()),
            filter: RulePathFilter::new_with_inferred(root, config, rule, inferred).ok()?,
            roots: super::super::target_roots_with_inferred(root, config, rule, inferred),
            skip: config.filesystem.skip_directories.iter().cloned().collect(),
        })
    }

    pub(crate) fn contains(&self, selector: &str) -> bool {
        if !is_literal_file(selector) {
            return false;
        }
        let path = crate::codebase::ts_resolver::normalize_path(&self.root.join(selector));
        self.includes_path(&path)
            && self.inventory.id_for_normalized_path(&path).is_none()
            && path.try_exists().is_ok_and(|exists| !exists)
    }

    pub(crate) fn includes_path(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
            && self.filter.is_match(path)
            && self.roots.iter().any(|root| {
                path.strip_prefix(root).is_ok_and(|relative| {
                    !crate::codebase::ts_source::is_under_skipped_dir(
                        root,
                        path,
                        &std::collections::HashSet::new(),
                    ) && !relative.components().any(|part| {
                        part.as_os_str()
                            .to_str()
                            .is_some_and(|name| self.skip.contains(name))
                    })
                })
            })
    }
}

pub(crate) fn findings(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    sources: &SourceStore,
    option: &str,
    entries: impl Iterator<Item = (String, String)>,
) -> Vec<super::RuleFinding> {
    findings_with_inferred(root, config, rule, sources, option, entries, None)
}

pub(crate) fn findings_with_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    sources: &SourceStore,
    option: &str,
    entries: impl Iterator<Item = (String, String)>,
    inferred: Option<&crate::codebase::config::InferredRoots>,
) -> Vec<super::RuleFinding> {
    if config.report_unused_allow_entries == DisableDirectivePolicy::Ignore
        || config.missing_allow_audit_scope_incomplete
        || !sources.inventory_is_complete()
    {
        return Vec::new();
    }
    let entries: Vec<_> = entries.collect();
    let audit = super::AllowAudit::new_files_with_inferred(
        root,
        config,
        rule,
        Some(sources),
        option,
        entries.len(),
        inferred,
    );
    audit.finish_files(
        |_| false,
        |i| entries[i].0.clone(),
        |i| entries[i].1.clone(),
    )
}

pub(crate) fn is_literal_file(file: &str) -> bool {
    !file.is_empty() && !file.contains(['*', '?', '[', ']', '{', '}', '\\'])
}
