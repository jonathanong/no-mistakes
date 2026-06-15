// Export-name discovery helpers used to pre-seed exported bindings/types before
// the AST visit, so references declared before their `export` statement are
// still treated as exported. Included into `extract.rs`.

/// Collects identifier references inside a default-export expression so they can
/// be pre-seeded as exported bindings.
#[derive(Default)]
struct DefaultExportIdentifierCollector {
    names: Vec<String>,
}

impl<'a> Visit<'a> for DefaultExportIdentifierCollector {
    fn visit_identifier_reference(&mut self, identifier: &oxc_ast::ast::IdentifierReference<'a>) {
        self.names.push(identifier.name.to_string());
    }

    // Do not descend into nested function/arrow bodies: their locals can shadow
    // outer bindings, and a reference inside an uninvoked callback does not make
    // the outer value part of the default export.
    fn visit_function(
        &mut self,
        _function: &oxc_ast::ast::Function<'a>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
    }

    fn visit_arrow_function_expression(
        &mut self,
        _arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
    }

    // Type-position identifiers (e.g. `{} as Lazy`) are not runtime value uses.
    fn visit_ts_type(&mut self, _ty: &oxc_ast::ast::TSType<'a>) {}
}

/// Identifier names referenced by a `export default <expr>` alias/wrapper, if any.
/// Seeding them as exported bindings lets a lazy binding declared before the
/// default export — `const Foo = dynamic(() => import('./Foo')); export default
/// Foo;` or `export default memo(Foo);` — keep its dynamic import reachable even
/// though the binding is visited first. Function/class/arrow defaults create
/// their own scope and are handled by the visitor, so they are skipped here.
fn later_default_export_value_names<'a>(program: &Program<'a>) -> Vec<String> {
    let Some(export) = program.body.iter().find_map(|statement| match statement {
        Statement::ExportDefaultDeclaration(export) => Some(export),
        _ => None,
    }) else {
        return Vec::new();
    };
    // Function/class/arrow defaults (including parenthesized ones) create their
    // own scope; collecting identifiers from their bodies would pre-seed shadowed
    // locals as exported and falsely mark their imports reachable.
    if matches!(
        export.declaration,
        ExportDefaultDeclarationKind::FunctionDeclaration(_)
            | ExportDefaultDeclarationKind::ClassDeclaration(_)
            | ExportDefaultDeclarationKind::ArrowFunctionExpression(_)
            | ExportDefaultDeclarationKind::FunctionExpression(_)
    ) || parenthesized_default_function(&export.declaration).is_some()
        || parenthesized_default_arrow(&export.declaration).is_some()
    {
        return Vec::new();
    }
    let mut collector = DefaultExportIdentifierCollector::default();
    collector.visit_export_default_declaration(export);
    collector.names
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
