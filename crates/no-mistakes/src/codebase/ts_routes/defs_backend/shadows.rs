fn statements_shadow_name(stmts: &[Statement], name: &str) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::VariableDeclaration(var_decl) => {
            variable_declaration_shadows_name(var_decl, name)
        }
        Statement::FunctionDeclaration(func) => func
            .id
            .as_ref()
            .map(|id| id.name.as_str() == name)
            .unwrap_or(false),
        Statement::ClassDeclaration(class) => class
            .id
            .as_ref()
            .map(|id| id.name.as_str() == name)
            .unwrap_or(false),
        _ => statement_has_function_scope_var_shadow(stmt, name),
    })
}

fn statement_has_function_scope_var_shadow(stmt: &Statement, name: &str) -> bool {
    match stmt {
        Statement::VariableDeclaration(var_decl) => {
            var_decl.kind == VariableDeclarationKind::Var
                && variable_declaration_shadows_name(var_decl, name)
        }
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .any(|stmt| statement_has_function_scope_var_shadow(stmt, name)),
        Statement::IfStatement(if_stmt) => {
            statement_has_function_scope_var_shadow(&if_stmt.consequent, name)
                || if_stmt
                    .alternate
                    .as_ref()
                    .map(|alt| statement_has_function_scope_var_shadow(alt, name))
                    .unwrap_or(false)
        }
        Statement::ForStatement(for_stmt) => {
            let init_shadows = matches!(
                &for_stmt.init,
                Some(ForStatementInit::VariableDeclaration(var_decl))
                    if var_decl.kind == VariableDeclarationKind::Var
                        && variable_declaration_shadows_name(var_decl, name)
            );
            init_shadows || statement_has_function_scope_var_shadow(&for_stmt.body, name)
        }
        Statement::ForInStatement(for_stmt) => {
            for_left_var_declaration_shadows_name(&for_stmt.left, name)
                || statement_has_function_scope_var_shadow(&for_stmt.body, name)
        }
        Statement::ForOfStatement(for_stmt) => {
            for_left_var_declaration_shadows_name(&for_stmt.left, name)
                || statement_has_function_scope_var_shadow(&for_stmt.body, name)
        }
        Statement::WhileStatement(while_stmt) => {
            statement_has_function_scope_var_shadow(&while_stmt.body, name)
        }
        Statement::DoWhileStatement(do_while_stmt) => {
            statement_has_function_scope_var_shadow(&do_while_stmt.body, name)
        }
        Statement::SwitchStatement(switch_stmt) => switch_stmt.cases.iter().any(|case| {
            case.consequent
                .iter()
                .any(|stmt| statement_has_function_scope_var_shadow(stmt, name))
        }),
        Statement::TryStatement(try_stmt) => {
            try_stmt
                .block
                .body
                .iter()
                .any(|stmt| statement_has_function_scope_var_shadow(stmt, name))
                || try_stmt
                    .handler
                    .as_ref()
                    .map(|handler| {
                        handler
                            .body
                            .body
                            .iter()
                            .any(|stmt| statement_has_function_scope_var_shadow(stmt, name))
                    })
                    .unwrap_or(false)
                || try_stmt
                    .finalizer
                    .as_ref()
                    .map(|finalizer| {
                        finalizer
                            .body
                            .iter()
                            .any(|stmt| statement_has_function_scope_var_shadow(stmt, name))
                    })
                    .unwrap_or(false)
        }
        _ => false,
    }
}

fn for_left_var_declaration_shadows_name(left: &ForStatementLeft, name: &str) -> bool {
    matches!(
        left,
        ForStatementLeft::VariableDeclaration(var_decl)
            if var_decl.kind == VariableDeclarationKind::Var
                && variable_declaration_shadows_name(var_decl, name)
    )
}

fn function_name_shadows_name(func: &oxc::ast::ast::Function, name: &str) -> bool {
    func.id
        .as_ref()
        .map(|id| id.name.as_str() == name)
        .unwrap_or(false)
}

fn variable_declaration_shadows_name(
    var_decl: &oxc::ast::ast::VariableDeclaration,
    name: &str,
) -> bool {
    var_decl
        .declarations
        .iter()
        .any(|decl| binding_pattern_contains_name(&decl.id, name))
}

fn params_shadow_name(params: &oxc::ast::ast::FormalParameters, name: &str) -> bool {
    params
        .items
        .iter()
        .any(|param| binding_pattern_contains_name(&param.pattern, name))
        || params
            .rest
            .as_ref()
            .map(|rest| binding_pattern_contains_name(&rest.rest.argument, name))
            .unwrap_or(false)
}

fn binding_pattern_contains_name(pattern: &BindingPattern, name: &str) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(id) => id.name.as_str() == name,
        BindingPattern::ObjectPattern(obj) => {
            obj.properties
                .iter()
                .any(|prop| binding_pattern_contains_name(&prop.value, name))
                || obj
                    .rest
                    .as_ref()
                    .map(|rest| binding_pattern_contains_name(&rest.argument, name))
                    .unwrap_or(false)
        }
        BindingPattern::ArrayPattern(arr) => {
            arr.elements
                .iter()
                .flatten()
                .any(|element| binding_pattern_contains_name(element, name))
                || arr
                    .rest
                    .as_ref()
                    .map(|rest| binding_pattern_contains_name(&rest.argument, name))
                    .unwrap_or(false)
        }
        BindingPattern::AssignmentPattern(assign) => {
            binding_pattern_contains_name(&assign.left, name)
        }
    }
}
