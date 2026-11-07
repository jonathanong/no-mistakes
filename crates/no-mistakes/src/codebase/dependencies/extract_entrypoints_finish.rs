fn finish_import_facts(mut collector: ImportCollector, program: &Program<'_>) -> ImportFacts {
    collector.record_finite_identifier_guards(&program.body, u32::MAX);
    collector.finalize_metadata_sources(program);
    collector.materialize_aggregate_aliases();
    collector.materialize_deferred_simple_aliases();
    collector.finish_forward_receiver_writes();
    collector.finish_foreign_receiver_writes();
    collector.finish_bounded_value_flow();

    let reassigned_callable_ids = collector
        .reassigned_callable_binding_ids
        .iter()
        .enumerate()
        .flat_map(|(binding_scope, names)| {
            names
                .iter()
                .map(move |callee| (binding_scope, callee.as_str()))
        })
        .filter_map(|(binding_scope, callee)| {
            let (binding, member) = callee
                .split_once('.')
                .map_or((callee, None), |(binding, member)| (binding, Some(member)));
            let callable_id = collector.callable_binding_at(binding_scope, binding)?;
            member.map_or(Some(callable_id), |member| {
                owner_member_id(
                    &collector.aggregate_callable_member_ids,
                    callable_id,
                    member,
                )
            })
        })
        .collect::<FxHashSet<_>>();
    let reassigned_callable_scopes = collector
        .callable_scope_ids
        .iter()
        .filter_map(|(id, scope)| {
            reassigned_callable_ids
                .contains(id)
                .then_some(scope.clone())
        })
        .collect::<FxHashSet<_>>();

    let mut exported_resource_roots: Vec<_> =
        collector.exported_resource_roots.into_iter().collect();
    exported_resource_roots.sort();
    let mut exported_resource_scopes: Vec<_> =
        collector.exported_resource_scopes.into_iter().collect();
    exported_resource_scopes.sort();
    let mut known_function_scopes: Vec<_> = collector.known_function_scopes.into_iter().collect();
    known_function_scopes.sort();
    let mut callable_scope_ids: Vec<_> = collector.callable_scope_ids.into_iter().collect();
    callable_scope_ids.sort();
    let mut callable_scopes: Vec<_> = collector.callable_scopes.into_iter().collect();
    callable_scopes.sort();
    let mut class_scopes: Vec<_> = collector.class_scopes.into_iter().collect();
    class_scopes.sort();
    let mut class_declaration_lines = collector.class_declaration_lines;
    class_declaration_lines.sort();
    let namespaces = finish_namespace_facts(collector.namespace);
    let mut callable_bindings = flatten_scope_map(collector.callable_bindings);
    callable_bindings.sort();
    let mut callable_binding_declared_at =
        flatten_scope_map(collector.callable_binding_declared_at);
    callable_binding_declared_at.sort();
    let mut class_member_callable_ids = flatten_owner_members(collector.class_member_callable_ids);
    class_member_callable_ids.sort();
    let mut static_getter_callable_ids = flatten_owner_members(collector.static_getter_member_ids);
    static_getter_callable_ids.sort();
    let mut static_setter_callable_ids = flatten_owner_members(collector.static_setter_member_ids);
    static_setter_callable_ids.sort();
    let mut lexical_scope_parents: Vec<_> = collector.lexical_scope_parents.into_iter().collect();
    lexical_scope_parents.sort_by_key(|(scope, _)| *scope);
    let callable_aliases = collector
        .callable_aliases
        .into_iter()
        .map(|binding| binding.alias)
        .collect();
    let exported_type_scopes = collector.exported_type_scopes;
    let mut exported_functions: Vec<_> = collector
        .exported_functions
        .into_iter()
        .filter(|scope| {
            !reassigned_callable_scopes.contains(scope)
                && (callable_scopes.contains(scope) || exported_type_scopes.contains(scope))
        })
        .collect();
    exported_functions.sort();
    ImportFacts {
        value_flow: collector.value_flow,
        imports: collector.imports,
        eager_require_specifiers: collector.eager_require_specifiers.into_iter().collect(),
        imported_bindings: collector.call_import_bindings,
        exported_bindings: collector.call_export_bindings,
        callable_aliases,
        star_reexport_specifiers: collector.star_reexport_specifiers,
        per_item_calls: collector.per_item_calls,
        function_calls: collector.function_calls,
        unreachable_call_ranges: normalize_unreachable_call_ranges(
            collector.unreachable_call_ranges,
        ),
        import_inactive_call_ranges: normalize_unreachable_call_ranges(
            collector
                .import_inactive_call_ranges
                .into_iter()
                .map(|(start, end)| (None, start, end))
                .collect(),
        )
        .into_iter()
        .map(|(_, start, end)| (start, end))
        .collect(),
        unknown_calls: collector.unknown_calls,
        symbol_references: collector.symbol_references,
        exported_functions,
        exported_resource_roots,
        exported_resource_scopes,
        known_function_scopes,
        callable_scope_ids,
        callable_bindings,
        callable_binding_declared_at,
        class_member_callable_ids,
        static_getter_callable_ids,
        static_setter_callable_ids,
        lexical_scope_parents,
        callable_scopes,
        class_scopes,
        class_declaration_lines,
        namespaces,
        has_unknown_top_level_call: collector.has_unknown_top_level_call,
    }
}
