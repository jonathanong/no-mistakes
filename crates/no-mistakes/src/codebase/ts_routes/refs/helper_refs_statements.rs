fn collect_helper_refs_from_statement<'a>(
    stmt: &'a Statement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    register_router_bindings_from_statement(stmt, router_bindings);
    register_helper_bindings_from_statement(stmt, helper_bindings, local_helpers);

    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            collect_helper_refs_from_expression(
                &expr_stmt.expression,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::ReturnStatement(ret_stmt) => {
            if let Some(expr) = &ret_stmt.argument {
                collect_helper_refs_from_expression(
                    expr,
                    source,
                    file,
                    router_bindings,
                    helper_bindings,
                    local_helpers,
                    refs,
                );
            }
        }
        Statement::BlockStatement(block) => {
            collect_helper_refs_from_block_statement(
                block,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::IfStatement(if_stmt) => {
            collect_helper_refs_from_if_statement(
                if_stmt,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::ForStatement(for_stmt) => {
            collect_helper_refs_from_for_statement(
                for_stmt,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::ForInStatement(for_stmt) => {
            collect_helper_refs_from_loop_body(
                (&for_stmt.right, &for_stmt.body),
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::ForOfStatement(for_stmt) => {
            collect_helper_refs_from_loop_body(
                (&for_stmt.right, &for_stmt.body),
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::WhileStatement(while_stmt) => {
            collect_helper_refs_from_loop_body(
                (&while_stmt.test, &while_stmt.body),
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::DoWhileStatement(do_while_stmt) => {
            collect_helper_refs_from_do_while_statement(
                do_while_stmt,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::SwitchStatement(switch_stmt) => {
            collect_helper_refs_from_switch_statement(
                switch_stmt,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::TryStatement(try_stmt) => {
            collect_helper_refs_from_try_statement(
                try_stmt,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::VariableDeclaration(var_decl) => {
            collect_helper_refs_from_var_declaration(
                var_decl,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::FunctionDeclaration(func) => {
            collect_helper_refs_from_function_body(
                func,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Statement::ExportNamedDeclaration(export) => collect_helper_refs_from_named_export(
            export,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        ),
        Statement::ExportDefaultDeclaration(export) => collect_helper_refs_from_default_export(
            export,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        ),
        _ => {}
    }
}
