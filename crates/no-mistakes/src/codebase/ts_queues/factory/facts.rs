pub fn find_queue_factory_facts(
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> (Option<u32>, Option<String>) {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        Path::new("queue-factory.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    find_queue_factory_facts_from_program(&ret.program, source, factory_specifier, factory_function)
}

pub fn find_queue_factory_facts_from_program(
    program: &Program<'_>,
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> (Option<u32>, Option<String>) {
    let walk = FactoryWalk {
        source,
        bindings: collect_factory_import_bindings(program),
        const_strings: collect_const_string_bindings(&program.body),
        specifier: factory_specifier,
        function: factory_function,
    };
    let mut line = None;
    let mut name = None;
    for stmt in &program.body {
        merge_factory_facts(&mut line, &mut name, factory_facts_in_stmt(stmt, &walk));
    }
    (line, name)
}

struct FactoryWalk<'a> {
    source: &'a str,
    bindings: HashMap<String, (String, String)>,
    const_strings: HashMap<String, String>,
    specifier: &'a str,
    function: &'a str,
}

fn merge_factory_facts(
    line: &mut Option<u32>,
    name: &mut Option<String>,
    found: (Option<u32>, Option<String>),
) {
    if line.is_none() {
        *line = found.0;
    }
    if name.is_none() {
        *name = found.1;
    }
}

fn collect_factory_import_bindings(program: &Program<'_>) -> HashMap<String, (String, String)> {
    let mut bindings = HashMap::new();
    for stmt in &program.body {
        let Statement::ImportDeclaration(import_decl) = stmt else {
            continue;
        };
        let Some(specifiers) = &import_decl.specifiers else {
            continue;
        };
        let src = import_decl.source.value.as_str();
        for specifier in specifiers {
            let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier else {
                continue;
            };
            bindings.insert(
                spec.local.name.as_str().to_string(),
                (src.to_string(), module_export_name_str(&spec.imported)),
            );
        }
    }
    bindings
}

fn collect_const_string_bindings(stmts: &[Statement<'_>]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for stmt in stmts {
        let var_decl = match stmt {
            Statement::VariableDeclaration(v) => v,
            Statement::ExportDeclaration(e) => {
                if let oxc_ast::ast::Declaration::VariableDeclaration(v) = &e.declaration {
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
            let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &decl.id else {
                continue;
            };
            if let Some(Expression::StringLiteral(s)) = &decl.init {
                map.insert(id.name.as_str().to_string(), s.value.as_str().to_string());
            }
        }
    }
    map
}

fn factory_facts_in_stmt(
    stmt: &Statement<'_>,
    walk: &FactoryWalk<'_>,
) -> (Option<u32>, Option<String>) {
    match stmt {
        Statement::ExpressionStatement(e) => factory_facts_in_expr(&e.expression, walk),
        Statement::VariableDeclaration(v) => factory_facts_in_var_decls(&v.declarations, walk),
        Statement::ExportDeclaration(e) => {
            let oxc_ast::ast::Declaration::VariableDeclaration(v) = &e.declaration else {
                return (None, None);
            };
            factory_facts_in_var_decls(&v.declarations, walk)
        }
        _ => (None, None),
    }
}

fn factory_facts_in_var_decls(
    declarations: &[oxc_ast::ast::VariableDeclarator<'_>],
    walk: &FactoryWalk<'_>,
) -> (Option<u32>, Option<String>) {
    let mut line = None;
    let mut name = None;
    for decl in declarations {
        let Some(init) = &decl.init else {
            continue;
        };
        merge_factory_facts(&mut line, &mut name, factory_facts_in_expr(init, walk));
    }
    (line, name)
}

fn factory_facts_in_expr(
    expr: &Expression<'_>,
    walk: &FactoryWalk<'_>,
) -> (Option<u32>, Option<String>) {
    match expr {
        Expression::CallExpression(call) => factory_facts_in_call(call, walk),
        Expression::TSAsExpression(ts_as) => factory_facts_in_expr(&ts_as.expression, walk),
        Expression::TSNonNullExpression(ts_nn) => factory_facts_in_expr(&ts_nn.expression, walk),
        _ => (None, None),
    }
}

fn factory_facts_in_call(
    call: &oxc_ast::ast::CallExpression<'_>,
    walk: &FactoryWalk<'_>,
) -> (Option<u32>, Option<String>) {
    if is_factory_callee(&call.callee, walk) {
        return (
            Some(byte_offset_to_line(walk.source, call.span.start as usize)),
            Some(resolve_queue_name(call, &walk.const_strings)),
        );
    }
    (nested_factory_call_line(call, walk), None)
}

fn is_factory_callee(callee: &Expression<'_>, walk: &FactoryWalk<'_>) -> bool {
    let Expression::Identifier(id) = callee else {
        return false;
    };
    walk.bindings
        .get(id.name.as_str())
        .is_some_and(|(src, imported)| src == walk.specifier && imported == walk.function)
}

fn nested_factory_call_line(
    call: &oxc_ast::ast::CallExpression<'_>,
    walk: &FactoryWalk<'_>,
) -> Option<u32> {
    call.arguments.iter().find_map(|arg| {
        let oxc_ast::ast::Argument::CallExpression(inner) = arg else {
            return None;
        };
        is_factory_callee(&inner.callee, walk)
            .then(|| byte_offset_to_line(walk.source, inner.span.start as usize))
    })
}

fn resolve_queue_name(
    call: &oxc_ast::ast::CallExpression<'_>,
    const_strings: &HashMap<String, String>,
) -> String {
    match call.arguments.first() {
        Some(oxc_ast::ast::Argument::StringLiteral(s)) => s.value.as_str().to_string(),
        Some(oxc_ast::ast::Argument::Identifier(id)) => const_strings
            .get(id.name.as_str())
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string()),
        _ => "<unknown>".to_string(),
    }
}

fn module_export_name_str(name: &ModuleExportName) -> String {
    name.name().as_str().to_string()
}
