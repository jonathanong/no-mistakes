use super::*;

pub(super) fn metadata_files(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Vec<PathBuf> {
    if !rule_enabled(config, FORBIDDEN_WORKSPACE_CLOSURE)
        && !rule_enabled(config, PRODUCTION_DEPENDENCY_DECLARATIONS)
    {
        return Vec::new();
    }
    let mut metadata_files = files.to_vec();
    metadata_files.extend(snapshot.paths_for(root).iter().cloned());
    metadata_files.sort();
    metadata_files.dedup();
    metadata_files
}
