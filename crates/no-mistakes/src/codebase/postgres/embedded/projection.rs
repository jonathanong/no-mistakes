use super::*;

pub(crate) fn extract_embedded_sql_from_program_with_projections(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    options: &EmbeddedSqlOptions,
    demand: (bool, bool),
) -> EmbeddedSqlFileFacts {
    record_embedded_collection!();
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
        demand,
    );
    #[cfg(feature = "test-instrumentation")]
    let _ = collected.lock_summary_passes;
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
        lock_helper_summaries: collected.lock_helper_summaries,
        imported_lock_helpers: collected.imported_lock_helpers,
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
