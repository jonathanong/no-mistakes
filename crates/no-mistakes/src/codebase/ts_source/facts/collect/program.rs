use super::super::{domain, TsFactContext, TsFactPlan, TsFileFacts};
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands;
use crate::codebase::ts_source::module_facts;
use crate::codebase::ts_symbols::extract_symbols_from_program;
use std::{path::Path, sync::Arc};

pub(crate) fn collect_file_facts_from_program(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    parse_error: Option<String>,
    stored_source: Option<Arc<str>>,
) -> TsFileFacts {
    collect_file_facts_from_program_with_semantic(
        path,
        plan,
        context,
        source,
        program,
        (parse_error, stored_source),
        None,
    )
}

pub(crate) fn collect_file_facts_from_program_with_semantic(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    source_state: (Option<String>, Option<Arc<str>>),
    semantic: Option<&oxc_semantic::SemanticBuilderReturn<'_>>,
) -> TsFileFacts {
    let (parse_error, stored_source) = source_state;
    let import_facts = if plan.needs_import_facts() {
        extract_import_facts_from_program_with_source_and_demands(
            program,
            source,
            plan.resources,
            plan.per_item_calls,
            plan.needs_value_flow(),
        )
    } else {
        Default::default()
    };
    let resources = if plan.resources {
        crate::codebase::ts_resources::extract(program, source)
    } else {
        Default::default()
    };
    let symbols = if plan.symbols {
        Some(Arc::new(extract_symbols_from_program(program, source)))
    } else {
        None
    };
    let domain = if plan.has_domain_facts() || plan.call_sites {
        domain::collect_domain_facts(program, path, source, plan, context)
    } else {
        domain::DomainFacts::default()
    };
    let effect_calls = if plan.effect_calls {
        domain::collect_effect_calls(&import_facts.function_calls, &context.effect_functions)
    } else {
        Vec::new()
    };
    let call_sites = domain.call_sites;
    let react_components = if plan.react {
        match context.visible_files.as_deref() {
            Some(visible) => crate::react_traits::analyze::file::analyze_program_from_visible(
                path,
                &context.root,
                source,
                program,
                visible,
            ),
            None => crate::react_traits::analyze::file::analyze_program(
                path,
                &context.root,
                source,
                program,
            ),
        }
        .components
    } else {
        Default::default()
    };
    TsFileFacts {
        module_bindings: (plan.module_bindings || plan.call_arguments).then(|| {
            Arc::new(match semantic {
                Some(built) => module_facts::extract_from_semantic(
                    &built.semantic,
                    &built.diagnostics,
                    plan.call_arguments && parse_error.is_none(),
                ),
                None => module_facts::extract_with_arguments(
                    program,
                    plan.call_arguments && parse_error.is_none(),
                ),
            })
        }),
        operational_error: None,
        parse_error,
        fatal_parse_error: false,
        source: stored_source.or_else(|| plan.source.then(|| Arc::<str>::from(source))),
        imports: import_facts.imports,
        eager_require_specifiers: import_facts.eager_require_specifiers,
        imported_bindings: import_facts.imported_bindings,
        exported_bindings: import_facts.exported_bindings,
        value_flow: import_facts.value_flow,
        callable_aliases: import_facts.callable_aliases,
        star_reexport_specifiers: import_facts.star_reexport_specifiers,
        per_item_calls: import_facts.per_item_calls,
        function_calls: import_facts.function_calls,
        unknown_calls: import_facts.unknown_calls,
        call_sites,
        resource_calls: resources.calls,
        resource_diagnostics: resources.diagnostics,
        symbol_references: import_facts.symbol_references,
        exported_functions: import_facts.exported_functions,
        exported_resource_roots: import_facts.exported_resource_roots,
        exported_resource_scopes: import_facts.exported_resource_scopes,
        known_function_scopes: import_facts.known_function_scopes,
        callable_scope_ids: import_facts.callable_scope_ids,
        callable_bindings: import_facts.callable_bindings,
        callable_binding_declared_at: import_facts.callable_binding_declared_at,
        class_member_callable_ids: import_facts.class_member_callable_ids,
        static_getter_callable_ids: import_facts.static_getter_callable_ids,
        static_setter_callable_ids: import_facts.static_setter_callable_ids,
        lexical_scope_parents: import_facts.lexical_scope_parents,
        callable_scopes: import_facts.callable_scopes,
        class_scopes: import_facts.class_scopes,
        class_declaration_lines: import_facts.class_declaration_lines,
        namespaces: import_facts.namespaces,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        symbols,
        route_refs: domain.route_refs,
        route_helpers: domain.route_helpers,
        route_helper_imports: domain.route_helper_imports,
        route_helper_refs: domain.route_helper_refs,
        backend_routes: domain.backend_routes,
        queue_usage: domain.queue_usage,
        queue_create_line: domain.queue_create_line,
        queue_name: domain.queue_name,
        queue_project: domain.queue_project,
        http_calls: domain.http_calls,
        process_spawns: domain.process_spawns,
        server_routes: domain.server_routes,
        react_components,
        effect_calls,
        rsc_environment: domain.rsc_environment,
        trpc_procedures: domain.trpc_procedures,
        trpc_calls: domain.trpc_calls,
    }
}
