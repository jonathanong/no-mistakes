use super::*;

pub(super) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_grouping_not_aggregation::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_grouping_not_aggregation::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}
