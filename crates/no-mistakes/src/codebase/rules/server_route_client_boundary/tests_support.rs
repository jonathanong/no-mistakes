use super::*;

pub(super) fn check_with_facts_for_aggregate(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let sources = super::super::source_store_for_files(shared.files());
    diagnostics_with_facts_for_aggregate(
        root,
        config,
        shared,
        &sources,
        inferred_roots,
        defer_suppression,
    )
    .map(|output| output.findings)
}
