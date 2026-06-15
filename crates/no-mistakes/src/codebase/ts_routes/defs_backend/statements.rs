fn collect_from_statement(
    stmt: &Statement,
    source: &str,
    register_object: &str,
    register_object_in_scope: bool,
    results: &mut Vec<(String, u32)>,
) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            collect_from_expression(
                &expr_stmt.expression,
                source,
                register_object,
                register_object_in_scope,
                results,
            );
        }
        Statement::VariableDeclaration(var_decl) => {
            let register_object_in_scope = register_object_in_scope
                && !var_decl
                    .declarations
                    .iter()
                    .any(|decl| binding_pattern_contains_name(&decl.id, register_object));
            for decl in &var_decl.declarations {
                if let Some(init) = &decl.init {
                    collect_from_expression(
                        init,
                        source,
                        register_object,
                        register_object_in_scope,
                        results,
                    );
                }
            }
        }
        Statement::BlockStatement(block) => {
            let register_object_in_scope =
                register_object_in_scope && !statements_shadow_name(&block.body, register_object);
            for s in &block.body {
                collect_from_statement(
                    s,
                    source,
                    register_object,
                    register_object_in_scope,
                    results,
                );
            }
        }
        Statement::FunctionDeclaration(func) => {
            collect_from_function_body(
                func,
                source,
                register_object,
                register_object_in_scope,
                results,
            );
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(decl) = &export.declaration {
                match decl {
                    oxc_ast::ast::Declaration::VariableDeclaration(var_decl) => {
                        let register_object_in_scope = register_object_in_scope
                            && !var_decl.declarations.iter().any(|decl| {
                                binding_pattern_contains_name(&decl.id, register_object)
                            });
                        for d in &var_decl.declarations {
                            if let Some(init) = &d.init {
                                collect_from_expression(
                                    init,
                                    source,
                                    register_object,
                                    register_object_in_scope,
                                    results,
                                );
                            }
                        }
                    }
                    oxc_ast::ast::Declaration::FunctionDeclaration(func) => {
                        collect_from_function_body(
                            func,
                            source,
                            register_object,
                            register_object_in_scope,
                            results,
                        );
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn collect_from_function_body(
    func: &oxc_ast::ast::Function,
    source: &str,
    register_object: &str,
    register_object_in_scope: bool,
    results: &mut Vec<(String, u32)>,
) {
    let Some(body) = &func.body else {
        return;
    };
    let register_object_in_scope = register_object_in_scope
        && !function_name_shadows_name(func, register_object)
        && !params_shadow_name(&func.params, register_object)
        && !statements_shadow_name(&body.statements, register_object);
    for s in &body.statements {
        collect_from_statement(
            s,
            source,
            register_object,
            register_object_in_scope,
            results,
        );
    }
}

fn collect_from_expression(
    expr: &Expression,
    source: &str,
    register_object: &str,
    register_object_in_scope: bool,
    results: &mut Vec<(String, u32)>,
) {
    if let Expression::CallExpression(call) = expr {
        if let Some(member) = call.callee.as_member_expression().filter(|member| {
            member
                .static_property_name()
                .is_some_and(|verb| HTTP_VERBS.contains(&verb))
        }) {
            let line = byte_offset_to_line(source, call.span.start as usize);
            if register_object_in_scope {
                if let Some(route_pattern) =
                    direct_route_arg(call, member.object(), register_object)
                        .or_else(|| extract_route_from_chain(member.object(), register_object))
                {
                    results.push((route_pattern, line));
                }
            }
            collect_from_expression(
                member.object(),
                source,
                register_object,
                register_object_in_scope,
                results,
            );
        }
        collect_from_expression(
            &call.callee,
            source,
            register_object,
            register_object_in_scope,
            results,
        );
        for arg in &call.arguments {
            if let Some(expr) = arg.as_expression() {
                collect_from_expression(
                    expr,
                    source,
                    register_object,
                    register_object_in_scope,
                    results,
                );
            }
        }
    }
}
