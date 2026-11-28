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

pub(super) fn column_check_constraints(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_column_check_constraints::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_column_check_constraints::check_with_files_and_sources(
            root, config, files, sources,
        ),
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

pub(super) fn materialized_view_shape(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_materialized_view_shape::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_materialized_view_shape::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}

pub(super) fn polymorphic_target(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_polymorphic_target::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_polymorphic_target::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

pub(super) fn run(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Option<Result<Vec<RuleFinding>>> {
    Some(match rule_id {
        POSTGRES_DUPLICATE_FUNCTION_BODY => {
            duplicate_function_body(root, config, files, sources, facts)
        }
        POSTGRES_COLUMN_CHECK_CONSTRAINTS => {
            column_check_constraints(root, config, files, sources, facts)
        }
        POSTGRES_POLYMORPHIC_TARGET => polymorphic_target(root, config, files, sources, facts),
        POSTGRES_REQUIRED_COMMENTS => required_comments(root, config, files, sources, facts),
        POSTGRES_MATERIALIZED_VIEW_SHAPE => {
            materialized_view_shape(root, config, files, sources, facts)
        }
        POSTGRES_COLUMN_REQUIRES_TRIGGER => {
            column_requires_trigger(root, config, files, sources, facts)
        }
        POSTGRES_CONFLICT_ORDERING => conflict_ordering(root, config, files, sources, facts),
        POSTGRES_LOCK_ORDERING => lock_ordering(root, config, files, sources, facts),
        _ => return None,
    })
}
