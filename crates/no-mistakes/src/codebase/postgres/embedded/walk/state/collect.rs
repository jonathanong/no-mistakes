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
    head_calls_only: bool,
) -> CollectedCalls {
    let track_order = !provisional.is_empty();
    let imported_namespace_names =
        super::super::namespace_imports::configured_namespaces(program, options);
    let imported_value_origins =
        super::super::namespace_imports::configured_values(program, options, bindings);
    let (imported_names, constant_namespaces) = imported::value_import_names(program);
    let mut visitor = ScopeVisitor {
        head_calls_only,
        source,
        configured_module: options.import_specifier.clone(),
        query_members: !options.import_specifier.is_empty()
            || options.executor_names.iter().any(|name| name == "query"),
        query_binding: options.executor_names.is_empty()
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
        functions: if head_calls_only {
            resolve::LocalFunctions::collect_heads(program, &options.trusted_sql_tags)
        } else {
            resolve::LocalFunctions::collect(program, source, &options.trusted_sql_tags)
        },
        sql_statement_types: sql_statement_type_bindings(program),
        unproven_member_query_spans: Vec::new(),
        literal_objects: super::super::const_literals::LiteralObjects::default(),
        declaring_const: false,
        clause_unions: crate::fx::fx_set(),
        exported_constants: Default::default(),
    };
    visitor.visit_program(program);
    visitor.downgrade_escaped_member_calls();
    if !head_calls_only {
        visitor.finish_const_splices();
    }
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
    CollectedCalls {
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
