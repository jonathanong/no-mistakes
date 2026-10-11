use super::*;

pub(super) fn shape_policy(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_sql_shape_policy::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_sql_shape_policy::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

#[cfg(test)]
mod tests;
