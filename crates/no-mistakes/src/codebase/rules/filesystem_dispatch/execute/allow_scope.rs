use crate::codebase::check_facts::PathMembership;
use crate::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use crate::config::v2::{DisableDirectivePolicy, NoMistakesConfig};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub(super) fn config<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &VisiblePathSnapshot,
    sources: &SourceStore,
) -> anyhow::Result<Cow<'a, NoMistakesConfig>> {
    if config.report_unused_allow_entries == DisableDirectivePolicy::Ignore
        || config.missing_allow_audit_scope_incomplete
        || !sources.inventory_is_complete()
    {
        return Ok(Cow::Borrowed(config));
    }
    let projects = super::super::FILESYSTEM_RULE_IDS
        .iter()
        .flat_map(|id| config.rule_applications(id))
        .flat_map(|rule| rule.projects.iter())
        .filter_map(|name| config.projects.get(name));
    let mut inferred = sources.inferred_roots_for_projects(root, projects);
    let roots = super::super::preserved::filesystem_rule_target_roots_with_inferred(
        root,
        config,
        super::super::FILESYSTEM_RULE_IDS,
        &mut inferred,
    )?;
    let expected = crate::codebase::ts_source::discover_files_preserving_roots_from_visible(
        root,
        &config.filesystem.skip_directories,
        &roots,
        &snapshot.paths_for(root),
    );
    let supplied = PathMembership::new(files);
    if expected.iter().all(|path| supplied.contains(path)) {
        Ok(Cow::Borrowed(config))
    } else {
        let mut narrowed = config.clone();
        narrowed.missing_allow_audit_scope_incomplete = true;
        Ok(Cow::Owned(narrowed))
    }
}
