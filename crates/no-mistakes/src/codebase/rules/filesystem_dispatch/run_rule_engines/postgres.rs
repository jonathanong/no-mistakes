use super::super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod generated_predicates;
mod naming;
mod prepared_schema;
mod schema;
mod shapes;
use schema::{
    column_requires_trigger, conflict_ordering, duplicate_function_body, lock_ordering,
    required_comments,
};
mod no_offset;

use naming::{
    array_columns, bounded_statements, column_naming, explicit_columns, finite_text, object_naming,
    required_predicates, status_lifecycle, table_shape,
};

pub(super) fn run(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Option<Result<Vec<RuleFinding>>> {
    let result = run_schema_rules(rule_id, root, config, files, sources, facts)
        .or_else(|| run_naming_and_query_rules(rule_id, root, config, files, sources, facts))?;
    Some(result.and_then(|mut findings| {
        if crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS.contains(&rule_id) {
            findings.extend(crate::codebase::rules::postgres_unmatched_executors::check(
                root, config, rule_id, files, sources, facts,
            )?);
            // Both producers have already deduplicated their own findings.
            // Equal public DTOs can name distinct physical variant columns.
            findings.sort();
        }
        Ok(findings)
    }))
}

fn run_schema_rules(
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
            prepared_schema::constraint_validate(root, config, files, sources, facts)
        }
        POSTGRES_NO_ADD_COLUMN => {
            prepared_schema::no_add_column(root, config, files, sources, facts)
        }
        POSTGRES_FK_INDEX => prepared_schema::fk_index(root, config, files, sources, facts),
        POSTGRES_REDUNDANT_INDEX => {
            prepared_schema::redundant_index(root, config, files, sources, facts)
        }
        POSTGRES_LOCK_ORDERING => lock_ordering(root, config, files, sources, facts),
        POSTGRES_IDENTIFIER_LENGTH => {
            prepared_schema::identifier_length(root, config, files, sources, facts)
        }
        POSTGRES_KEY_COLUMN_TYPES => {
            prepared_schema::key_column_types(root, config, files, sources, facts)
        }
        POSTGRES_REQUIRE_FK_ON_DELETE => {
            prepared_schema::require_fk_on_delete(root, config, files, sources, facts)
        }
        POSTGRES_REQUIRE_NAMED_CONSTRAINTS => {
            prepared_schema::require_named_constraints(root, config, files, sources, facts)
        }
        POSTGRES_SQL_STATEMENT_POLICY => {
            prepared_schema::sql_statement_policy(root, config, files, sources, facts)
        }
        _ => return None,
    })
}

fn run_naming_and_query_rules(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Option<Result<Vec<RuleFinding>>> {
    Some(match rule_id {
        POSTGRES_BOUNDED_STATEMENTS => bounded_statements(root, config, files, sources, facts),
        POSTGRES_EXPLICIT_COLUMNS => explicit_columns(root, config, files, sources, facts),
        POSTGRES_GENERATED_COLUMN_PREDICATES => {
            generated_predicates::run(root, config, files, sources, facts)
        }
        POSTGRES_NO_GENERATED_COLUMN_WRITES => match facts {
            Some(facts) => postgres_no_generated_column_writes::check_with_files_sources_and_facts(
                root, config, files, sources, facts,
            ),
            None => postgres_no_generated_column_writes::check_with_files_and_sources(
                root, config, files, sources,
            ),
        },
        POSTGRES_NO_OFFSET => no_offset::run(root, config, files, sources, facts),
        POSTGRES_ARRAY_COLUMNS => array_columns(root, config, files, sources, facts),
        POSTGRES_FINITE_TEXT_COLUMNS => finite_text(root, config, files, sources, facts),
        POSTGRES_COLUMN_NAMING => column_naming(root, config, files, sources, facts),
        POSTGRES_OBJECT_NAMING => object_naming(root, config, files, sources, facts),
        POSTGRES_REQUIRE_QUERY_ANNOTATION => match facts {
            Some(facts) => postgres_require_query_annotation::check_with_files_sources_and_facts(
                root, config, files, sources, facts,
            ),
            None => postgres_require_query_annotation::check_with_files_and_sources(
                root, config, files, sources,
            ),
        },
        POSTGRES_REQUIRED_PREDICATES => required_predicates(root, config, files, sources, facts),
        POSTGRES_SQL_SHAPE_POLICY => shapes::shape_policy(root, config, files, sources, facts),
        POSTGRES_STATUS_WITH_LIFECYCLE_TIMESTAMPS => {
            status_lifecycle(root, config, files, sources, facts)
        }
        POSTGRES_TABLE_SHAPE => table_shape(root, config, files, sources, facts),
        POSTGRES_IDEMPOTENT_INSERT => {
            postgres_idempotent_insert::check_with_files_and_sources(root, config, files, sources)
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
