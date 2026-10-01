use super::super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
        POSTGRES_REQUIRED_COMMENTS => required_comments(root, config, files, sources, facts),
        POSTGRES_COLUMN_REQUIRES_TRIGGER => {
            column_requires_trigger(root, config, files, sources, facts)
        }
        POSTGRES_CONFLICT_ORDERING => conflict_ordering(root, config, files, sources, facts),
        POSTGRES_CONSTRAINT_VALIDATE => {
            postgres_constraint_validate::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_NO_ADD_COLUMN => {
            postgres_no_add_column::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_FK_INDEX => {
            postgres_fk_index::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_REDUNDANT_INDEX => {
            postgres_redundant_index::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_NO_GENERATED_COLUMN_WRITES => {
            postgres_no_generated_column_writes::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        POSTGRES_LOCK_ORDERING => lock_ordering(root, config, files, sources, facts),
        POSTGRES_NO_OFFSET => {
            postgres_no_offset::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_REQUIRE_FK_ON_DELETE => {
            postgres_require_fk_on_delete::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        POSTGRES_REQUIRE_NAMED_CONSTRAINTS => {
            postgres_require_named_constraints::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        POSTGRES_REQUIRE_QUERY_ANNOTATION => {
            postgres_require_query_annotation::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        POSTGRES_REQUIRED_PREDICATES => {
            postgres_required_predicates::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_SQL_SHAPE_POLICY => {
            postgres_sql_shape_policy::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_SQL_STATEMENT_POLICY => {
            postgres_sql_statement_policy::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        POSTGRES_IDEMPOTENT_INSERT => {
            postgres_idempotent_insert::check_with_files_and_sources(root, config, files, sources)
        }
        _ => return None,
    })
}

fn duplicate_function_body(
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

fn required_comments(
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

fn column_requires_trigger(
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

fn conflict_ordering(
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

fn lock_ordering(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codebase::rules::postgres_column_requires_trigger::RULE_ID as COLUMN_RULE;
    use crate::codebase::rules::postgres_conflict_ordering::RULE_ID as CONFLICT_RULE;
    use crate::codebase::rules::postgres_duplicate_function_body::RULE_ID as DUPLICATE_RULE;
    use crate::codebase::rules::postgres_required_comments::RULE_ID as COMMENTS_RULE;
    use crate::codebase::ts_source::FileInventory;

    #[test]
    fn missing_facts_use_the_file_checkers() {
        let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[]))));
        let config = NoMistakesConfig::default();
        let files: &[PathBuf] = &[];
        for rule_id in [DUPLICATE_RULE, COMMENTS_RULE, COLUMN_RULE, CONFLICT_RULE] {
            assert!(run(rule_id, Path::new("."), &config, files, &sources, None).is_some());
        }
    }
}
