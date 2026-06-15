// Export-name discovery helpers used to pre-seed exported bindings/types before
// the AST visit, so references declared before their `export` statement are
// still treated as exported. Included into `extract.rs`.

/// The identifier name of a `export default <ident>` alias, if any. Seeding it as
/// an exported binding lets `const Foo = dynamic(() => import('./Foo')); export
/// default Foo;` treat the dynamic import as reachable even though the binding is
/// declared before the default export statement.
fn later_default_export_value_name<'a>(program: &Program<'a>) -> Option<String> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::Identifier(identifier) => {
                Some(identifier.name.to_string())
            }
            _ => None,
        }
    })
}

fn later_named_value_exports<'a>(
    program: &Program<'a>,
    local_type_names: &HashSet<String>,
) -> Vec<String> {
    let mut exports = Vec::new();
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() || export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() {
                continue;
            }
            if let Some(name) = module_export_name_name(&specifier.local) {
                if local_type_names.contains(name) {
                    continue;
                }
                exports.push(name.to_string());
            }
        }
    }
    exports
}

fn later_named_type_exports<'a>(
    program: &Program<'a>,
    local_type_names: &HashSet<String>,
) -> Vec<String> {
    let mut exports = Vec::new();
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() {
            continue;
        }
        for specifier in &export.specifiers {
            if let Some(name) = module_export_name_name(&specifier.local) {
                if !export.export_kind.is_type()
                    && !specifier.export_kind.is_type()
                    && !local_type_names.contains(name)
                {
                    continue;
                }
                exports.push(name.to_string());
            }
        }
    }
    exports
}

fn local_type_declaration_names<'a>(program: &Program<'a>) -> HashSet<String> {
    program
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::TSTypeAliasDeclaration(decl) => Some(decl.id.name.as_str().to_string()),
            Statement::TSInterfaceDeclaration(decl) => Some(decl.id.name.as_str().to_string()),
            _ => None,
        })
        .collect()
}
