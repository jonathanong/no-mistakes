fn collect_route_helper_imports<'a>(program: &'a Program<'a>) -> Vec<RouteHelperImport> {
    let mut imports = Vec::new();
    let mut default_aliases = Vec::new();
    let mut named_aliases = Vec::new();
    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(import) => {
                if import.import_kind.is_type() {
                    continue;
                }
                let source = import.source.value.as_str();
                let Some(specifiers) = &import.specifiers else {
                    continue;
                };
                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier)
                            if !specifier.import_kind.is_type() =>
                        {
                            imports.push(RouteHelperImport {
                                local: specifier.local.name.as_str().to_string(),
                                imported: specifier.imported.name().to_string(),
                                source: source.to_string(),
                            });
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            imports.push(RouteHelperImport {
                                local: specifier.local.name.as_str().to_string(),
                                imported: "default".to_string(),
                                source: source.to_string(),
                            });
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            imports.push(RouteHelperImport {
                                local: specifier.local.name.as_str().to_string(),
                                imported: "*".to_string(),
                                source: source.to_string(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            Statement::ExportNamedDeclaration(export)
                if export.source.is_some() && !export.export_kind.is_type() =>
            {
                let source = export.source.as_ref().expect("checked source").value.as_str();
                for specifier in &export.specifiers {
                    if specifier.export_kind.is_type() {
                        continue;
                    }
                    imports.push(RouteHelperImport {
                        local: specifier.exported.name().to_string(),
                        imported: specifier.local.name().to_string(),
                        source: source.to_string(),
                    });
                }
            }
            Statement::ExportAllDeclaration(export) if !export.export_kind.is_type() => {
                imports.push(RouteHelperImport {
                    local: export
                        .exported
                        .as_ref()
                        .map(|name| name.name().to_string())
                        .unwrap_or_else(|| "*".to_string()),
                    imported: "*".to_string(),
                    source: export.source.value.as_str().to_string(),
                });
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(alias) = default_export_alias_name(&export.declaration) {
                    default_aliases.push(alias.to_string());
                }
            }
            Statement::ExportNamedDeclaration(export)
                if export.source.is_none() && export.declaration.is_none() && !export.export_kind.is_type() =>
            {
                for specifier in &export.specifiers {
                    if !specifier.export_kind.is_type() {
                        named_aliases.push((
                            specifier.local.name().to_string(),
                            specifier.exported.name().to_string(),
                        ));
                    }
                }
            }
            Statement::ExportNamedDeclaration(export)
                if export.source.is_none() && !export.export_kind.is_type() =>
            {
                if let Some(declaration) = export.declaration.as_ref() {
                    imports.extend(exported_imported_helper_wrapper(declaration, &imports));
                }
            }
            _ => {}
        }
    }
    for alias in default_aliases {
        let forwarded = imports
            .iter()
            .find(|import| import.local == alias)
            .cloned();
        if let Some(import) = forwarded {
            imports.push(RouteHelperImport {
                local: "default".to_string(),
                imported: import.imported,
                source: import.source,
            });
        }
    }
    for (local, exported) in named_aliases {
        let forwarded = imports
            .iter()
            .find(|import| import.local == local)
            .cloned();
        if let Some(import) = forwarded {
            imports.push(RouteHelperImport {
                local: exported,
                imported: import.imported,
                source: import.source,
            });
        }
    }
    imports.sort_by(|a, b| (&a.local, &a.imported, &a.source).cmp(&(&b.local, &b.imported, &b.source)));
    imports.dedup();
    imports
}

fn default_export_alias_name<'a>(
    declaration: &'a oxc_ast::ast::ExportDefaultDeclarationKind<'a>,
) -> Option<&'a str> {
    match declaration {
        oxc_ast::ast::ExportDefaultDeclarationKind::Identifier(id) => Some(id.name.as_str()),
        oxc_ast::ast::ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            default_export_expression_alias_name(&parenthesized.expression)
        }
        other => other
            .as_expression()
            .and_then(default_export_expression_alias_name),
    }
}

fn default_export_expression_alias_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::ParenthesizedExpression(parenthesized) => {
            default_export_expression_alias_name(&parenthesized.expression)
        }
        _ => None,
    }
}

fn collect_route_helper_refs_from_program<'a>(
    program: &'a Program<'a>,
    source: &str,
    file: &str,
    helpers: &[RouteHelper],
    imports: &[RouteHelperImport],
) -> Vec<RouteHelperRef> {
    let mut router_bindings = collect_import_bindings(&program.body);
    collect_router_bindings_for_scope(&program.body, &mut router_bindings);
    let mut helper_bindings = collect_route_helper_bindings(helpers, imports);
    let local_helpers = helpers
        .iter()
        .map(|helper| helper.name.clone())
        .collect::<HashSet<_>>();

    let mut refs = Vec::new();
    for stmt in &program.body {
        collect_helper_refs_from_statement(
            stmt,
            source,
            file,
            &mut router_bindings,
            &mut helper_bindings,
            &local_helpers,
            &mut refs,
        );
    }
    refs.sort_by(|a, b| {
        (&a.file, a.line, &a.callee, &a.wrapper_pattern).cmp(&(
            &b.file,
            b.line,
            &b.callee,
            &b.wrapper_pattern,
        ))
    });
    refs.dedup();
    refs
}
