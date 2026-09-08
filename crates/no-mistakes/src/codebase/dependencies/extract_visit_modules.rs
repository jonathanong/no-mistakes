fn visit_import_declaration_with_scope(
    collector: &mut ImportCollector,
    import: &ImportDeclaration<'_>,
) {
    let kind = import_declaration_kind(import);
    let side_effect_only = import
        .specifiers
        .as_ref()
        .is_none_or(|specifiers| specifiers.is_empty());
    collector.push_with_side_effect(
        import.source.value.as_str(),
        kind,
        import.span.start as usize,
        side_effect_only,
        false,
    );
    collector.record_imported_bindings(import);
}

fn visit_export_all_declaration_with_scope(
    collector: &mut ImportCollector,
    export: &ExportAllDeclaration<'_>,
) {
    let kind = if export.export_kind.is_type() {
        ImportKind::Type
    } else {
        ImportKind::Static
    };
    collector.push_reexport(
        export.source.value.as_str(),
        kind,
        export.span.start as usize,
    );
    // `export * as namespace from` has one concrete exported name. It is
    // not a transparent `export *` forwarding edge.
    if !export.export_kind.is_type() && export.exported.is_none() {
        collector
            .star_reexport_specifiers
            .push(export.source.value.to_string());
    }
}

fn visit_import_expression_with_scope(collector: &mut ImportCollector, import: &ImportExpression<'_>) {
    if let Some(specifier) = static_import_specifier(&import.source) {
        collector.push(&specifier, ImportKind::Dynamic, import.span.start as usize);
    }
    walk::walk_import_expression(collector, import);
}

fn visit_ts_import_type_with_scope(collector: &mut ImportCollector, import: &TSImportType<'_>) {
    collector.push(
        import.source.value.as_str(),
        ImportKind::Type,
        import.span.start as usize,
    );
    walk::walk_ts_import_type(collector, import);
}
