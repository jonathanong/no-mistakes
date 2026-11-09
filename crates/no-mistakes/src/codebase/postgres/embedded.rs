use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, Program};
use oxc_span::SourceType;
use std::path::Path;

#[cfg(not(test))]
macro_rules! record_embedded_collection {
    () => {};
}
#[cfg(test)]
#[macro_use]
mod tests;
mod bindings;
mod call;
pub use call::{EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlVariant, MAX_EMBEDDED_SQL_VARIANTS};
mod dml_kind;
pub(crate) mod helpers;
mod options;
mod placeholders;
pub(crate) use placeholders::{
    count_placeholders, publish_placeholders_with_positions, renumber_placeholders,
    INTERNAL_PLACEHOLDER,
};
pub(crate) mod relative;
pub(crate) mod undeclared;
mod unproven;
pub use undeclared::{
    summarize_undeclared_executor_types, UndeclaredExecutorType, UndeclaredExecutorTypeSummary,
};
pub(crate) use unproven::retain_unproven_spans;
pub(super) mod scoped_bindings;
pub(crate) mod site_identity;
mod source_positions;
pub use source_positions::EmbeddedSqlSourcePosition;
mod tags;
mod text;
pub(crate) use tags::matches_trusted_sql_import;
pub(super) use text::unpublished_sql_text;
pub use text::{executed_query_text, sql_text};
pub(crate) mod walk;
pub use bindings::{executor_bindings, is_database_call};
pub(crate) use dml_kind::{
    recovered_sql_may_select, recovered_sql_may_write_columns, recovered_sql_needs_insert_check,
};
pub use options::{EmbeddedSqlOptions, TrustedSqlTag};
pub(crate) use relative::{
    package_name, package_root_for_specifier, project_relative_scoped_facts, PendingRelativeScope,
};
pub fn extract_embedded_sql_from_source(
    path: &Path,
    source: &str,
    options: &EmbeddedSqlOptions,
) -> EmbeddedSqlFileFacts {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = crate::ast::parse(path, &allocator, source, source_type);
    extract_embedded_sql_from_program(path, &parsed.program, source, options)
}
pub fn extract_embedded_sql_from_program(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    options: &EmbeddedSqlOptions,
) -> EmbeddedSqlFileFacts {
    extract_embedded_sql_from_program_with_demand(path, program, source, options, false)
}

pub(crate) fn extract_embedded_sql_from_program_with_demand(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    options: &EmbeddedSqlOptions,
    head_calls_only: bool,
) -> EmbeddedSqlFileFacts {
    extract_embedded_sql_from_program_with_projections(
        path,
        program,
        source,
        options,
        (head_calls_only, false),
    )
}
mod projection;
pub(crate) use projection::extract_embedded_sql_from_program_with_projections;

mod fragment_debug;
pub use fragment_debug::{EmbeddedSqlFileFacts, EmbeddedSqlFragment};

pub(super) fn first_call_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        other => other.as_expression(),
    }
}

#[cfg(test)]
mod append_mutation_tests;
#[cfg(test)]
mod chain_composition_tests;
#[cfg(test)]
mod chain_dynamic_prefix_tests;
#[cfg(test)]
mod chain_reassignment_destructuring_tests;
#[cfg(test)]
mod chain_tests;
#[cfg(test)]
mod compose_classification_tests;
#[cfg(test)]
mod conditional_clause_tests;
#[cfg(test)]
mod configured_sql_tag_tests;
#[cfg(test)]
mod const_splice_tests;
#[cfg(test)]
mod default_export_constants_tests;
#[cfg(test)]
mod detached_tests;
#[cfg(test)]
mod dml_kind_tests;
#[cfg(test)]
mod executor_obligations_tests;
#[cfg(test)]
mod head_demand_tests;
#[cfg(test)]
mod helper_body_tests;
#[cfg(test)]
mod imported_sql_tag_tests;
#[cfg(test)]
mod nested_sql_fragment_tests;
#[cfg(test)]
mod resolution_gaps_tests;
#[cfg(test)]
mod scoped_bindings_tests;
#[cfg(test)]
mod trusted_sql_tag_tests;
#[cfg(test)]
mod undeclared_tests;
#[cfg(test)]
#[path = "embedded/variants/tests.rs"]
mod variants_tests;

#[cfg(test)]
mod transaction_tests;

#[cfg(test)]
mod exact_origins_tests;
