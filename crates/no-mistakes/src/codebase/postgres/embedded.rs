use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, Program};
use oxc_span::SourceType;
use std::path::Path;

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
/// Parse `source` and extract executor SQL call sites.
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
/// Extract executor SQL call sites from an already-parsed program.
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
    let bindings = executor_bindings(program, options);
    let mut executor_bindings: Vec<String> = bindings
        .iter()
        .filter(|binding| binding.as_str() != self::bindings::MEMBER_QUERY_OPT_IN)
        .cloned()
        .collect();
    executor_bindings.sort();
    let mut scoped = scoped_bindings::collect(program, options);
    let mut collected = walk::collect_calls(
        program,
        source,
        &bindings,
        &scoped.executors,
        &scoped.spans,
        options,
        head_calls_only,
    );
    scoped.prune_unused_diagnostic_candidates(&collected.undeclared_executor_types);
    let mut fragments = collected.fragments;
    for fragment in &mut fragments {
        if let Some(sql_text) = fragment.sql_text.take() {
            let (sql_text, positions) = placeholders::publish_placeholders_with_positions(sql_text);
            fragment.sql_text = Some(sql_text);
            fragment.recovered_placeholder_positions = positions;
        }
    }
    let (matched_factory_names, matched_type_names) =
        scoped_bindings::matched_names(program, options);
    EmbeddedSqlFileFacts {
        matched_factory_names,
        matched_type_names,
        path: path.to_path_buf(),
        executor_bindings,
        calls: collected.calls,
        undeclared_executor_types: collected.undeclared_executor_types,
        call_spans: collected.call_spans,
        exported_helpers: collected.exported_helpers,
        exported_programs: collected.exported_programs,
        exported_constants: collected.exported_constants,
        pending_imported: collected.pending_imported,
        imported_call_lines: Default::default(),
        source_comment_spans: program
            .comments
            .iter()
            .map(|comment| (comment.span.start, comment.span.end))
            .collect(),
        fragments,
        fragment_variants: collected.fragment_variants,
        fragment_sites: collected.fragment_sites,
        pending_relative: PendingRelativeScope {
            candidates: scoped.candidates,
            spans: scoped.spans,
            calls: std::mem::take(&mut collected.pending_calls),
            diagnostic_member_sequences: collected.diagnostic_member_sequences,
            confirmed_order: std::mem::take(&mut collected.confirmed_order),
            call_spans: collected.pending_spans,
        },
        unproven_member_query_spans: collected.unproven_member_query_spans,
        postgres_rule_ids: Vec::new(),
        record_unproven_spans: true,
    }
}

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
mod dml_kind_tests;
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
mod tests;
#[cfg(test)]
mod trusted_sql_tag_tests;
#[cfg(test)]
#[path = "embedded/variants/tests.rs"]
mod variants_tests;

#[cfg(test)]
mod undeclared_tests;
