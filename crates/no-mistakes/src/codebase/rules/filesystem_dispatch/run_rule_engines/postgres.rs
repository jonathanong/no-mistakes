use super::super::*;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn run(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Option<Result<Vec<RuleFinding>>> {
    Some(match rule_id {
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
        POSTGRES_LOCK_ORDERING => {
            postgres_lock_ordering::check_with_files_and_sources(root, config, files, sources)
        }
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
        POSTGRES_SQL_STATEMENT_POLICY => {
            postgres_sql_statement_policy::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        _ => return None,
    })
}
