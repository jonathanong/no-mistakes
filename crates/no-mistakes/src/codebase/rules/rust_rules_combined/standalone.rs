use super::*;

pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    rule_id: &str,
) -> Result<Vec<RuleFinding>> {
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    let root = normalized_root.as_path();
    let mut selected = config.clone();
    selected.rules.retain(|rule| rule.rule == rule_id);
    let mut roots = std::collections::BTreeSet::new();
    for rule in selected.rule_applications(rule_id) {
        let opts: MaxLinesOptions = rule.try_rule_options()?;
        roots.extend(selection::normalize_roots(
            root,
            &super::super::target_roots(root, config, rule),
            &opts.roots,
        ));
    }
    let mut files: Vec<_> = roots
        .into_iter()
        .flat_map(|root| {
            crate::codebase::ts_source::discover_with_extensions(
                &root,
                &config.filesystem.skip_directories,
                &["rs"],
            )
        })
        .collect();
    files.sort();
    files.dedup();
    let sources = super::super::source_store_for_files(&files);
    check_with_files_sources_and_deferred_suppression(root, &selected, &files, &sources, false)
}
