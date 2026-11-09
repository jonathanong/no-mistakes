use super::super::{executors::MemberQueryProof, imported, resolve};
use super::{CollectedCalls, ScopeVisitor};
use crate::codebase::postgres::embedded::{
    bindings::sql_statement_type_bindings, helpers, relative::PendingRelativeSpan,
    scoped_bindings::ScopedExecutors, EmbeddedSqlOptions,
};
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::{BTreeMap, HashSet};

pub(crate) fn collect_calls<'a>(
    program: &Program<'a>,
    source: &'a str,
    bindings: &'a HashSet<String>,
    scoped: &'a ScopedExecutors,
    provisional: &'a [PendingRelativeSpan],
    options: &EmbeddedSqlOptions,
    demand: (bool, bool),
) -> CollectedCalls {
    let (head_calls_only, exact_sql_origins) = demand;
    let track_order = !provisional.is_empty();
    let imported_namespace_names =
        super::super::namespace_imports::configured_namespaces(program, options);
    let imported_value_origins =
        super::super::namespace_imports::configured_values(program, options, bindings);
    let (imported_names, constant_namespaces) = imported::value_import_names(program);
    let vitest_it_imports = if head_calls_only {
        Default::default()
    } else {
        super::super::finite_identifiers::vitest_it_imports(program)
    };
    let mut visitor = ScopeVisitor {
        #[cfg(feature = "test-instrumentation")]
        lock_summary_passes: 0,
        applied_lock_helper_spans: Default::default(),
        awaited_lock_helper_calls: Default::default(),
        imported_lock_helpers: Vec::new(),
        lock_helpers: super::super::lock_helpers::collect(
            program,
            options.track_transaction_helpers,
            source,
        ),
        incomplete_transaction_owners: Default::default(),
        exited_functions: Default::default(),
        stable_primitives: Default::default(),
        stable_parameter_fields: Default::default(),
        scalar_literals: Default::default(),
        head_calls_only,
        exact_sql_origins,
        source,
        configured_module: options.import_specifier.clone(),
        executor_options: options,
        query_members: !options.import_specifier.is_empty()
            || options.executor_names.iter().any(|name| name == "query"),
        query_binding: options.uses_legacy_executor_defaults()
            || options.executor_names.iter().any(|name| name == "query"),
        bindings,
        scoped,
        provisional,
        scopes: Vec::new(),
        undeclared_scopes: Vec::new(),
        undeclared_executor_types: Vec::new(),
        executor_methods: super::super::undeclared::methods(options),
        relative_proofs: Vec::new(),
        namespace_proofs: Vec::new(),
        imported_namespace_names,
        imported_value_origins,
        member_query_proof: MemberQueryProof::default(),
        builder_aliases: crate::fx::FxHashMap::default(),
        argument_effects: Vec::new(),
        argument_frames: 0,
        builder_reference_exemptions: Vec::new(),
        consumed_links: Default::default(),
        statement_calls: Default::default(),
        executor_alias_refs: Default::default(),
        executor_reference_spans: Default::default(),
        executor_direct_refs: Default::default(),
        executor_invoked_refs: Default::default(),
        calls: Vec::new(),
        call_spans: Vec::new(),
        imported_names,
        constant_namespaces,
        iife: None,
        imported_sites: Default::default(),
        pending_spans: BTreeMap::new(),
        pending_calls: Vec::new(),
        confirmed_order: Vec::new(),
        next_seq: 0,
        track_order,
        fragments: Vec::new(),
        fragment_variants: Vec::new(),
        fragment_sites: Vec::new(),
        suppress_nested_builder_fragments: 0,
        variant_paths: vec![Vec::new()],
        append_decision_offs: Default::default(),
        variant_guard_owner: None,
        control_depth: 0,
        loop_depth: 0,
        function_scopes: Vec::new(),
        var_scopes: Vec::new(),
        functions: if head_calls_only {
            resolve::LocalFunctions::collect_heads(program, &options.trusted_sql_tags)
        } else {
            resolve::LocalFunctions::collect(program, source, &options.trusted_sql_tags)
        },
        sql_statement_types: sql_statement_type_bindings(program),
        unproven_member_query_spans: Vec::new(),
        transaction_options: Default::default(),
        literal_objects: super::super::const_literals::LiteralObjects::default(),
        vitest_it_imports,
        finite_cases: Vec::new(),
        guarded_cases: Vec::new(),
        set_intrinsic_safe: head_calls_only
            || !(source.contains("Set") || source.contains("\\u"))
            || super::super::guarded_sql::intrinsic_safe(program),
        declaring_const: false,
        clause_unions: crate::fx::fx_set(),
        exported_constants: Default::default(),
        plain: Default::default(),
    };
    visitor.visit_program(program);
    visitor.flush_plain_returns();
    if options.track_transaction_helpers {
        visitor.mark_lock_helper_definitions();
    }
    visitor.downgrade_escaped_member_calls();
    if !head_calls_only {
        visitor.finish_const_splices();
    }
    visitor.finish_executor_references();
    super::super::observability::record(&visitor.calls);
    let sites = std::mem::take(&mut *visitor.imported_sites.borrow_mut()).into_list();
    let pending_imported = helpers::finalize::finalize(
        helpers::finalize::Walked {
            calls: &mut visitor.calls,
            call_spans: &visitor.call_spans,
            pending_calls: &mut visitor.pending_calls,
            pending_spans: &visitor.pending_spans,
            fragments: &mut visitor.fragments,
            fragment_variants: &mut visitor.fragment_variants,
        },
        &sites,
    );
    let diagnostic_member_sequences = visitor
        .member_query_proof
        .calls
        .iter()
        .filter(|call| call.sql_looking && visitor.pending_spans.contains_key(&call.sequence))
        .map(|call| call.sequence)
        .collect();
    let lock_helper_summaries = if options.track_transaction_helpers {
        visitor.lock_helper_summaries()
    } else {
        Default::default()
    };
    CollectedCalls {
        #[cfg(feature = "test-instrumentation")]
        lock_summary_passes: visitor.lock_summary_passes,
        lock_helper_summaries,
        imported_lock_helpers: visitor.imported_lock_helpers,
        undeclared_executor_types: visitor.undeclared_executor_types,
        calls: visitor.calls,
        call_spans: visitor.call_spans,
        exported_helpers: visitor.functions.exported_helpers(),
        exported_programs: visitor.functions.take_programs(),
        exported_constants: visitor.exported_constants,
        pending_imported,
        pending_spans: visitor.pending_spans,
        fragments: visitor.fragments,
        fragment_variants: visitor.fragment_variants,
        fragment_sites: visitor.fragment_sites,
        pending_calls: visitor.pending_calls,
        diagnostic_member_sequences,
        confirmed_order: visitor.confirmed_order,
        unproven_member_query_spans: visitor.unproven_member_query_spans,
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
mod summary_pass_tests;
