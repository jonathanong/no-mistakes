use super::*;
use crate::codebase::rules;

/// Select an application's files from the request-owned visible-file inventory.
pub(crate) fn selected_rule_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    selected_rule_files_with_roots(root, config, rule, all_files).map(|(files, _)| files)
}

pub(crate) fn selected_rule_files_with_roots(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (files, target_roots) = rule_scope_files_with_roots(root, config, rule, all_files);
    let files = rules::path_filter::filter_rule_files(root, config, rule, &files)?;
    Ok((files, target_roots))
}

/// Preserve companion-file visibility before an application's include/exclude filter.
pub(crate) fn rule_scope_files_with_roots(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let target_roots = rules::target_roots(root, config, rule);
    let skip = rules::skip_dir_set(config);
    let files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| rules::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
        .cloned()
        .collect();
    (files, target_roots)
}

pub(crate) fn selected_rule_files_with_sources(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<PathBuf>> {
    source_selection(root, config, rule, all_files, sources, |_| true).map(|(files, _)| files)
}

pub(crate) fn selected_rule_files_with_roots_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    source_selection(root, config, rule, all_files, sources, |_| true)
}

pub(crate) fn selected_rule_files_matching_with_sources(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    matches: impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>> {
    source_selection(root, config, rule, all_files, sources, matches).map(|(files, _)| files)
}

fn source_selection(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    matches: impl Fn(&Path) -> bool,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut inferred = if sources.inventory_is_complete() {
        sources.inferred_roots_for_projects(
            root,
            rule.projects
                .iter()
                .filter_map(|name| config.projects.get(name)),
        )
    } else {
        Default::default()
    };
    let roots = rules::target_roots_with_inferred(root, config, rule, &mut inferred);
    let skip = rules::skip_dir_set(config);
    let filter =
        rules::path_filter::RulePathFilter::new_with_inferred(root, config, rule, &mut inferred)?;
    let files = all_files
        .iter()
        .filter(|path| {
            matches(path)
                && rules::file_allowed_by_roots_and_skip(root, &skip, path, &roots)
                && filter.is_match(path)
        })
        .cloned()
        .collect();
    Ok((files, roots))
}
