use super::*;
use crate::codebase::ts_source::discover_files;

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    let root = normalized_root.as_path();
    let roots: std::collections::BTreeSet<_> = config
        .rule_applications(RULE_ID)
        .into_iter()
        .flat_map(|rule| super::super::target_roots(root, config, rule))
        .collect();
    let mut files: Vec<_> = roots
        .into_iter()
        .flat_map(|root| discover_files(&root, &config.filesystem.skip_directories))
        .collect();
    files.sort();
    files.dedup();
    check_with_files(root, config, &files)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::super::analysis_policy::AnalysisDiagnostics> {
    super::super::file_diagnostics::read_applications::<Options>(
        super::super::file_diagnostics::FileRuleInputs {
            root,
            config,
            files: all_files,
            sources,
            defer_suppression,
        },
        RULE_ID,
        |options, files, _, _| scan(root, options, files, sources),
    )
}
