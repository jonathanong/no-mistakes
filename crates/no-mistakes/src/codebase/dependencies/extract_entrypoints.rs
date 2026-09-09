pub fn extract_imports_from_program<'a>(program: &Program<'a>) -> Vec<ExtractedImport> {
    extract_import_facts_from_program(program).imports
}

pub fn extract_import_facts_from_program<'a>(program: &Program<'a>) -> ImportFacts {
    extract_import_facts_from_program_with_source(program, "")
}

pub fn extract_import_facts_from_program_with_source<'a>(
    program: &Program<'a>,
    source: &str,
) -> ImportFacts {
    extract_import_facts_from_program_with_source_and_resource_roots(program, source, true)
}

pub(crate) fn extract_import_facts_from_program_with_source_and_resource_roots<'a>(
    program: &Program<'a>,
    source: &str,
    collect_resource_roots: bool,
) -> ImportFacts {
    let mut collector = ImportCollector {
        line_starts: import_line_starts(source),
        collect_resource_roots,
        ..ImportCollector::default()
    };
    // Establish the file lexical environment before visiting expressions so a
    // later declaration cannot make an earlier global-looking call resolve to
    // a built-in. This is collection from the already parsed Program, not a
    // second source pass.
    collector.local_stack.push(HashSet::new());
    collector.lexical_scope_ids.push(0);
    collector.lexical_scope_parents.insert(0, None);
    collector.next_lexical_scope_id = 1;
    predeclare_program_value_bindings(&mut collector, program);
    let local_type_names = local_type_declaration_names(program);
    collector
        .exported_functions
        .extend(later_named_value_exports(program, &local_type_names));
    collector
        .exported_functions
        .extend(later_default_export_value_names(program));
    collector
        .later_exported_type_names
        .extend(later_named_type_exports(program, &local_type_names));
    collector.visit_program(program);
    collector.materialize_aggregate_aliases();

    let reassigned_callable_ids = collector
        .reassigned_callable_binding_ids
        .iter()
        .filter_map(|(binding_scope, callee)| {
            let (binding, member) = callee
                .split_once('.')
                .map_or((callee.as_str(), None), |(binding, member)| {
                    (binding, Some(member))
                });
            let callable_id = *collector
                .callable_bindings
                .get(&(*binding_scope, binding.to_string()))?;
            member.map_or(Some(callable_id), |member| {
                collector.aggregate_callable_member_ids.iter().find_map(
                    |(class_id, candidate, member_id)| {
                        (*class_id == callable_id && candidate == member).then_some(*member_id)
                    },
                )
            })
        })
        .collect::<HashSet<_>>();
    let reassigned_callable_scopes = collector
        .callable_scope_ids
        .iter()
        .filter_map(|(id, scope)| reassigned_callable_ids.contains(id).then_some(scope.clone()))
        .collect::<HashSet<_>>();

    let mut exported_resource_roots: Vec<_> =
        collector.exported_resource_roots.into_iter().collect();
    exported_resource_roots.sort();
    let mut exported_resource_scopes: Vec<_> =
        collector.exported_resource_scopes.into_iter().collect();
    exported_resource_scopes.sort();
    let mut known_function_scopes: Vec<_> = collector.known_function_scopes.into_iter().collect();
    known_function_scopes.sort();
    let mut callable_scope_ids: Vec<_> = collector
        .callable_scope_ids
        .into_iter()
        .collect();
    callable_scope_ids.sort();
    let mut callable_scopes: Vec<_> = collector
        .callable_scopes
        .into_iter()
        .collect();
    callable_scopes.sort();
    let mut class_scopes: Vec<_> = collector.class_scopes.into_iter().collect();
    class_scopes.sort();
    let mut callable_bindings: Vec<_> = collector
        .callable_bindings
        .into_iter()
        .map(|((scope, name), id)| (scope, name, id))
        .collect();
    callable_bindings.sort();
    let mut class_member_callable_ids: Vec<_> =
        collector.class_member_callable_ids.into_iter().collect();
    class_member_callable_ids.sort();
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
        imports: collector.imports,
        imported_bindings: collector.call_import_bindings,
        exported_bindings: collector.call_export_bindings,
        callable_aliases,
        star_reexport_specifiers: collector.star_reexport_specifiers,
        function_calls: collector.function_calls,
        unknown_calls: collector.unknown_calls,
        symbol_references: collector.symbol_references,
        exported_functions,
        exported_resource_roots,
        exported_resource_scopes,
        known_function_scopes,
        callable_scope_ids,
        callable_bindings,
        class_member_callable_ids,
        lexical_scope_parents,
        callable_scopes,
        class_scopes,
        has_unknown_top_level_call: collector.has_unknown_top_level_call,
    }
}

fn import_line_starts(source: &str) -> Vec<u32> {
    if source.is_empty() {
        return Vec::new();
    }
    let mut starts = vec![0u32];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push((index + 1) as u32);
        }
    }
    starts
}

#[cfg(test)]
#[path = "extract_entrypoints_import_line_tests.rs"]
mod import_line_tests;

fn import_line_at(line_starts: &[u32], byte_offset: usize) -> u32 {
    if line_starts.is_empty() {
        return 1;
    }
    let offset = byte_offset as u32;
    match line_starts.binary_search(&offset) {
        Ok(index) => index as u32 + 1,
        Err(index) => index as u32,
    }
}
