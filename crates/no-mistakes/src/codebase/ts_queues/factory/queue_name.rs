fn collect_const_string_bindings(stmts: &[Statement]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for stmt in stmts {
        let var_decl = match stmt {
            Statement::VariableDeclaration(v) => v,
            Statement::ExportNamedDeclaration(e) => {
                if let Some(oxc_ast::ast::Declaration::VariableDeclaration(v)) = &e.declaration {
                    v
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        if var_decl.kind != oxc_ast::ast::VariableDeclarationKind::Const {
            continue;
        }
        for decl in &var_decl.declarations {
            let name = match &decl.id {
                oxc_ast::ast::BindingPattern::BindingIdentifier(id) => {
                    id.name.as_str().to_string()
                }
                _ => continue,
            };
            if let Some(Expression::StringLiteral(s)) = &decl.init {
                map.insert(name, s.value.as_str().to_string());
            }
        }
    }
    map
}

/// Parse `source` and return the queue name from a `createQueue(name, ...)` call.
/// Resolves top-level `const NAME = "..."` bindings when the first argument is an identifier.
/// Returns `Some("<unknown>")` when the call is found but the name cannot be statically resolved.
pub fn find_queue_name(
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = crate::ast::parse(Path::new("queue-name.ts"), &allocator, source, source_type);
    find_queue_name_from_program(&ret.program, factory_specifier, factory_function)
}

pub fn find_queue_name_from_program<'a>(
    program: &Program<'a>,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<String> {
    let mut bindings: HashMap<String, (String, String)> = HashMap::new();
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import_decl) = stmt {
            let src = import_decl.source.value.as_str();
            if let Some(specifiers) = &import_decl.specifiers {
                for specifier in specifiers {
                    if let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier {
                        let imported_name = module_export_name_str(&spec.imported);
                        let local_name = spec.local.name.as_str().to_string();
                        bindings.insert(local_name, (src.to_string(), imported_name));
                    }
                }
            }
        }
    }

    let const_strings = collect_const_string_bindings(&program.body);

    for stmt in &program.body {
        if let Some(name) = find_queue_name_in_stmt(
            stmt,
            &bindings,
            &const_strings,
            factory_specifier,
            factory_function,
        ) {
            return Some(name);
        }
    }
    None
}

fn find_queue_name_in_stmt(
    stmt: &Statement,
    bindings: &HashMap<String, (String, String)>,
    const_strings: &HashMap<String, String>,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<String> {
    match stmt {
        Statement::ExpressionStatement(e) => find_queue_name_in_expr(
            &e.expression,
            bindings,
            const_strings,
            factory_specifier,
            factory_function,
        ),
        Statement::VariableDeclaration(v) => {
            for decl in &v.declarations {
                if let Some(init) = &decl.init {
                    if let Some(name) = find_queue_name_in_expr(
                        init,
                        bindings,
                        const_strings,
                        factory_specifier,
                        factory_function,
                    ) {
                        return Some(name);
                    }
                }
            }
            None
        }
        Statement::ExportNamedDeclaration(e) => {
            if let Some(oxc_ast::ast::Declaration::VariableDeclaration(v)) = &e.declaration {
                for d in &v.declarations {
                    if let Some(init) = &d.init {
                        if let Some(name) = find_queue_name_in_expr(
                            init,
                            bindings,
                            const_strings,
                            factory_specifier,
                            factory_function,
                        ) {
                            return Some(name);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn find_queue_name_in_expr(
    expr: &Expression,
    bindings: &HashMap<String, (String, String)>,
    const_strings: &HashMap<String, String>,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<String> {
    match expr {
        Expression::CallExpression(call_expr) => {
            let callee_name = match &call_expr.callee {
                Expression::Identifier(id) => Some(id.name.as_str()),
                _ => None,
            }?;
            if let Some((src, imported)) = bindings.get(callee_name) {
                if src == factory_specifier && imported == factory_function {
                    let resolved = match call_expr.arguments.first() {
                        Some(oxc_ast::ast::Argument::StringLiteral(s)) => {
                            s.value.as_str().to_string()
                        }
                        Some(oxc_ast::ast::Argument::Identifier(id)) => const_strings
                            .get(id.name.as_str())
                            .cloned()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        _ => "<unknown>".to_string(),
                    };
                    return Some(resolved);
                }
            }
            None
        }
        Expression::TSAsExpression(ts_as) => find_queue_name_in_expr(
            &ts_as.expression,
            bindings,
            const_strings,
            factory_specifier,
            factory_function,
        ),
        Expression::TSNonNullExpression(ts_nn) => find_queue_name_in_expr(
            &ts_nn.expression,
            bindings,
            const_strings,
            factory_specifier,
            factory_function,
        ),
        _ => None,
    }
}
