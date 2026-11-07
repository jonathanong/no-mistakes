impl ImportCollector {
    fn record_keyset_page_reads(&mut self, statement: &oxc_ast::ast::WhileStatement<'_>) {
        if !self.collect_per_item_calls || self.inside_per_item(statement.span) {
            return;
        }
        let Expression::Identifier(cursor) = &statement.test else {
            return;
        };
        let name = cursor.name.as_str();
        let Some(scope) = self.callee_binding_scope(name) else {
            return;
        };
        if !self
            .mutable_cursor_bindings
            .get(&scope)
            .is_some_and(|names| names.contains(name))
            && !self
                .value_parameter_indexes
                .get(&scope)
                .is_some_and(|parameters| parameters.contains_key(name))
        {
            return;
        }
        let Statement::BlockStatement(body) = &statement.body else {
            return;
        };
        let mut guards = CursorGuards::default();
        guards.visit_statement(&statement.body);
        if guards.opaque || guards.bindings.contains(name) {
            return;
        }
        for pair in body.body.windows(2) {
            let Statement::VariableDeclaration(declaration) = &pair[0] else {
                continue;
            };
            if declaration.kind != VariableDeclarationKind::Const
                || declaration.declarations.len() != 1
            {
                continue;
            }
            let declarator = &declaration.declarations[0];
            let Some(result) = binding_identifier_name(&declarator.id) else {
                continue;
            };
            if guards.writes.contains_key(result) {
                continue;
            }
            let Some(Expression::AwaitExpression(awaited)) = &declarator.init else {
                continue;
            };
            let Expression::CallExpression(call) = &awaited.argument else {
                continue;
            };
            let Statement::ExpressionStatement(expression) = &pair[1] else {
                continue;
            };
            let Expression::AssignmentExpression(assignment) = &expression.expression else {
                continue;
            };
            if assignment.operator != oxc_ast::ast::AssignmentOperator::Assign
                || !matches!(&assignment.left, AssignmentTarget::AssignmentTargetIdentifier(id) if id.name == name)
                || guards.writes.get(name).map(Vec::as_slice) != Some(&[assignment.span.start])
            {
                continue;
            }
            let Some(key) = keyset_last_member(&assignment.right, result) else {
                continue;
            };
            if keyset_call(call, name, key)
                && !keyset_escapes(
                    &statement.body,
                    call.span.start,
                    assignment.span.start,
                    name,
                    result,
                )
            {
                self.record_bounded_batch_call(call.span.start);
            }
        }
    }
}

fn keyset_last_member<'a>(expression: &'a Expression<'a>, result: &str) -> Option<&'a str> {
    let member = match expression {
        Expression::StaticMemberExpression(member) => member.as_ref(),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => member.as_ref(),
            _ => return None,
        },
        _ => return None,
    };
    let Expression::CallExpression(call) = &member.object else {
        return None;
    };
    let Expression::StaticMemberExpression(at) = &call.callee else {
        return None;
    };
    if at.property.name != "at"
        || call.arguments.len() != 1
        || !matches!(&at.object, Expression::Identifier(id) if id.name == result)
    {
        return None;
    }
    let Some(Expression::UnaryExpression(negative)) = call.arguments[0].as_expression() else {
        return None;
    };
    if negative.operator != oxc_ast::ast::UnaryOperator::UnaryNegation
        || !matches!(&negative.argument, Expression::NumericLiteral(number) if number.value == 1.0)
    {
        return None;
    }
    Some(member.property.name.as_str())
}

fn keyset_call(call: &CallExpression<'_>, cursor: &str, key: &str) -> bool {
    if call.optional || call.arguments.len() != 2 {
        return false;
    }
    let Some(Expression::ArrayExpression(arguments)) = call.arguments[1].as_expression() else {
        return false;
    };
    if arguments.elements.len() != 1
        || !matches!(arguments.elements[0].as_expression(), Some(Expression::Identifier(id)) if id.name == cursor)
    {
        return false;
    }
    let Some(sql) = call.arguments[0]
        .as_expression()
        .and_then(keyset_literal_sql)
    else {
        return false;
    };
    keyset_sql(&sql, key)
}

fn keyset_escapes(body: &Statement<'_>, page: u32, advance: u32, cursor: &str, rows: &str) -> bool {
    struct Escapes<'b> {
        page: u32,
        advance: u32,
        cursor: &'b str,
        rows: &'b str,
        escaped: bool,
    }
    impl<'a> Visit<'a> for Escapes<'_> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            if call.span.start != self.page {
                walk::walk_call_expression(self, call);
            }
        }
        fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
            if assignment.span.start != self.advance {
                walk::walk_assignment_expression(self, assignment);
            }
        }
        fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
            self.escaped |= identifier.name == self.cursor || identifier.name == self.rows;
        }
    }
    let mut escapes = Escapes {
        page,
        advance,
        cursor,
        rows,
        escaped: false,
    };
    escapes.visit_statement(body);
    escapes.escaped
}
