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
        source: source.to_string(),
        collect_resource_roots,
        ..ImportCollector::default()
    };
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

    let mut exported_resource_roots: Vec<_> =
        collector.exported_resource_roots.into_iter().collect();
    exported_resource_roots.sort();
    let mut exported_resource_scopes: Vec<_> =
        collector.exported_resource_scopes.into_iter().collect();
    exported_resource_scopes.sort();
    let callable_scopes = collector.callable_scopes;
    let exported_type_scopes = collector.exported_type_scopes;
    let mut exported_functions: Vec<_> = collector
        .exported_functions
        .into_iter()
        .filter(|scope| callable_scopes.contains(scope) || exported_type_scopes.contains(scope))
        .collect();
    exported_functions.sort();
    ImportFacts {
        imports: collector.imports,
        function_calls: collector.function_calls,
        symbol_references: collector.symbol_references,
        exported_functions,
        exported_resource_roots,
        exported_resource_scopes,
        unknown_callers: collector.unknown_callers,
        has_unknown_top_level_call: collector.has_unknown_top_level_call,
    }
}
