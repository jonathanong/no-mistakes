use super::*;

pub(super) fn duplicate_function_body(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_duplicate_function_body::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_duplicate_function_body::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}

pub(super) fn required_comments(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_required_comments::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_required_comments::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

pub(super) fn column_requires_trigger(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_column_requires_trigger::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_column_requires_trigger::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}

pub(super) fn conflict_ordering(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_conflict_ordering::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_conflict_ordering::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

pub(super) fn lock_ordering(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_lock_ordering::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_lock_ordering::check_with_files_and_sources(root, config, files, sources),
    }
}
