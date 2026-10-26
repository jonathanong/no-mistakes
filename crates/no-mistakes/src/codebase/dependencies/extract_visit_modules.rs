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
        crate::ast::js_literal(&import.source).as_ref(),
        kind,
        import.span.start as usize,
        side_effect_only,
        false,
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
        crate::ast::js_literal(&export.source).as_ref(),
        kind,
        export.span.start as usize,
    );
    // `export * as namespace from` has one concrete exported name. Retain a
    // namespace marker rather than treating it as a transparent star source:
    // a later named import of that name may call one member of its source.
    if !export.export_kind.is_type() {
        if let Some(exported) = export.exported.as_ref().and_then(module_export_name_name) {
            collector.call_export_bindings.push(ExportedBinding {
                specifier: Some(crate::ast::js_literal(&export.source).into_owned()),
                // `*` cannot be an ECMAScript export name. It distinguishes
                // the namespace object from a normal named re-export.
                local: "*".to_string(),
                exported: exported.to_string(),
            });
        } else {
            collector
                .star_reexport_specifiers
                .push(crate::ast::js_literal(&export.source).into_owned());
        }
    }
}

fn visit_import_expression_with_scope(
    collector: &mut ImportCollector,
    import: &ImportExpression<'_>,
) {
    if let Some(specifier) = static_import_specifier(&import.source) {
        collector.push(&specifier, ImportKind::Dynamic, import.span.start as usize);
    } else {
        collector.push_computed(
            &computed_import_specifier(&import.source),
            ImportKind::Dynamic,
            import.span.start as usize,
        );
    }
    walk::walk_import_expression(collector, import);
}

fn record_runtime_require_import(
    collector: &mut ImportCollector,
    call: &CallExpression<'_>,
    kind: ImportKind,
) {
    let Some(first) = call.arguments.first() else {
        return;
    };
    let Some(expr) = first.as_expression() else {
        collector.push_computed("<computed>", kind, call.span.start as usize);
        return;
    };
    if let Some(specifier) = static_import_specifier(expr) {
        collector.push(&specifier, kind, call.span.start as usize);
        return;
    }
    collector.push_computed(
        &computed_import_specifier(expr),
        kind,
        call.span.start as usize,
    );
}

fn computed_import_specifier(expr: &Expression<'_>) -> String {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) => ident.name.to_string(),
        Expression::TemplateLiteral(template) => {
            let mut specifier = String::new();
            for (i, quasi) in template.quasis.iter().enumerate() {
                specifier.push_str(crate::ast::template_quasi_text(quasi).as_ref());
                if i < template.expressions.len() {
                    specifier.push_str("${}");
                }
            }
            if specifier.is_empty() {
                "<computed>".to_string()
            } else {
                specifier
            }
        }
        _ => "<computed>".to_string(),
    }
}

fn visit_ts_import_type_with_scope(collector: &mut ImportCollector, import: &TSImportType<'_>) {
    collector.push(
        crate::ast::js_literal(&import.source).as_ref(),
        ImportKind::Type,
        import.span.start as usize,
    );
    walk::walk_ts_import_type(collector, import);
}
