use super::*;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_source::SourceStore;

pub(crate) fn required_sources(
    root: &Path,
    config: &NoMistakesConfig,
    candidates: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<PathBuf>> {
    let imports = required_import_fact_files(root, config, candidates)?;
    let mut required = Vec::new();
    for path in imports.iter() {
        if path.file_name().is_some_and(|name| name == "package.json") {
            sources.parse_json_path(path).map_err(|error| {
                CheckInputFailure::source(
                    path.clone(),
                    format!(
                        "failed to load workspace manifest {}: {error}",
                        path.display()
                    ),
                )
            })?;
        } else if is_indexable(path) {
            required.push(path.clone());
        }
    }
    Ok(required)
}
