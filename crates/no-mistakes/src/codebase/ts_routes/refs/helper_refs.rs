fn collect_route_helper_imports<'a>(program: &'a Program<'a>) -> Vec<RouteHelperImport> {
    let mut imports = Vec::new();
    for stmt in &program.body {
        let Statement::ImportDeclaration(import) = stmt else {
            continue;
        };
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
    imports.sort_by(|a, b| {
        a.local
            .cmp(&b.local)
            .then_with(|| a.imported.cmp(&b.imported))
            .then_with(|| a.source.cmp(&b.source))
    });
    imports.dedup();
    imports
}

fn collect_route_helper_refs_from_program<'a>(
    program: &'a Program<'a>,
    source: &str,
    file: &str,
) -> Vec<RouteHelperRef> {
    let mut router_bindings = collect_import_bindings(&program.body);
    collect_router_bindings_for_scope(&program.body, &mut router_bindings);

    let mut refs = Vec::new();
    for stmt in &program.body {
        collect_helper_refs_from_statement(stmt, source, file, &mut router_bindings, &mut refs);
    }
    refs.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.callee.cmp(&b.callee))
    });
    refs.dedup();
    refs
}
