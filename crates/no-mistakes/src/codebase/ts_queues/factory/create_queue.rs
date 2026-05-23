pub fn find_create_queue_line(
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<u32> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    find_create_queue_line_from_program(&ret.program, source, factory_specifier, factory_function)
}

pub fn find_create_queue_line_from_program<'a>(
    program: &Program<'a>,
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<u32> {
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

    for stmt in &program.body {
        if let Some(line) = check_stmt_for_create_queue(
            stmt,
            source,
            &bindings,
            factory_specifier,
            factory_function,
        ) {
            return Some(line);
        }
    }

    None
}

fn module_export_name_str(name: &ModuleExportName) -> String {
    name.name().as_str().to_string()
}

fn check_stmt_for_create_queue(
    stmt: &Statement,
    source: &str,
    bindings: &HashMap<String, (String, String)>,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<u32> {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => check_expr_for_create_queue(
            &expr_stmt.expression,
            source,
            bindings,
            factory_specifier,
            factory_function,
        ),
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                if let Some(init) = &decl.init {
                    if let Some(line) = check_expr_for_create_queue(
                        init,
                        source,
                        bindings,
                        factory_specifier,
                        factory_function,
                    ) {
                        return Some(line);
                    }
                }
            }
            None
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(decl) = &export.declaration {
                match decl {
                    oxc::ast::ast::Declaration::VariableDeclaration(var_decl) => {
                        for d in &var_decl.declarations {
                            if let Some(init) = &d.init {
                                if let Some(line) = check_expr_for_create_queue(
                                    init,
                                    source,
                                    bindings,
                                    factory_specifier,
                                    factory_function,
                                ) {
                                    return Some(line);
                                }
                            }
                        }
                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn check_expr_for_create_queue(
    expr: &Expression,
    source: &str,
    bindings: &HashMap<String, (String, String)>,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<u32> {
    match expr {
        Expression::CallExpression(call_expr) => {
            let callee_name = match &call_expr.callee {
                Expression::Identifier(id) => Some(id.name.as_str()),
                _ => None,
            };

            if let Some(name) = callee_name {
                if let Some((src, imported)) = bindings.get(name) {
                    if src == factory_specifier && imported == factory_function {
                        let line = byte_offset_to_line(source, call_expr.span.start as usize);
                        return Some(line);
                    }
                }
            }

            if let Some(line) = call_expr.arguments.iter().find_map(|arg| {
                let oxc::ast::ast::Argument::CallExpression(inner) = arg else {
                    return None;
                };
                let Expression::Identifier(id) = &inner.callee else {
                    return None;
                };
                bindings.get(id.name.as_str()).and_then(|(src, imported)| {
                    (src == factory_specifier && imported == factory_function)
                        .then(|| byte_offset_to_line(source, inner.span.start as usize))
                })
            }) {
                return Some(line);
            }
            None
        }
        Expression::TSAsExpression(ts_as) => check_expr_for_create_queue(
            &ts_as.expression,
            source,
            bindings,
            factory_specifier,
            factory_function,
        ),
        Expression::TSNonNullExpression(ts_nn) => check_expr_for_create_queue(
            &ts_nn.expression,
            source,
            bindings,
            factory_specifier,
            factory_function,
        ),
        _ => None,
    }
}
