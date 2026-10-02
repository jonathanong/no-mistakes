use super::*;

pub(super) fn constraint_validate(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_constraint_validate::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_constraint_validate::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

pub(super) fn no_add_column(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_no_add_column::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_no_add_column::check_with_files_and_sources(root, config, files, sources),
    }
}

pub(super) fn fk_index(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_fk_index::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_fk_index::check_with_files_and_sources(root, config, files, sources),
    }
}

pub(super) fn redundant_index(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_redundant_index::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_redundant_index::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

pub(super) fn identifier_length(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_identifier_length::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_identifier_length::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

pub(super) fn require_fk_on_delete(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_require_fk_on_delete::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_require_fk_on_delete::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}

pub(super) fn require_named_constraints(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_require_named_constraints::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_require_named_constraints::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}

pub(super) fn sql_statement_policy(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_sql_statement_policy::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_sql_statement_policy::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}
