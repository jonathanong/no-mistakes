impl ImportCollector {
    fn record_mutable_cursor_bindings(&mut self, declaration: &VariableDeclaration<'_>) {
        if !self.collect_per_item_calls
            || !matches!(
                declaration.kind,
                VariableDeclarationKind::Let | VariableDeclarationKind::Var
            )
        {
            return;
        }
        for declarator in &declaration.declarations {
            let Some(name) = binding_identifier_name(&declarator.id) else {
                continue;
            };
            if let Some(scope) = self.callee_binding_scope(name) {
                self.mutable_cursor_bindings
                    .entry(scope)
                    .or_default()
                    .insert(name.to_string());
            }
        }
    }

    fn record_capped_cursor_reads(&mut self, statement: &oxc_ast::ast::ForStatement<'_>) {
        if !self.collect_per_item_calls
            || self
                .per_item_ranges
                .get(&self.current_function_id())
                .into_iter()
                .flatten()
                .any(|range| {
                    range.span.start <= statement.span.start && statement.span.end <= range.span.end
                })
        {
            return;
        }
        let Some((counter, bound)) = capped_cursor_counter(statement) else {
            return;
        };
        let Statement::BlockStatement(body) = &statement.body else {
            return;
        };
        let mut guards = CursorGuards::default();
        guards.visit_statement(&statement.body);
        if guards.opaque
            || guards.writes.contains_key(counter)
            || guards.bindings.contains(counter)
            || bound.is_some_and(|name| {
                guards.writes.contains_key(name) || guards.bindings.contains(name)
            })
        {
            return;
        }
        for (index, item) in body.body.iter().enumerate() {
            let Statement::VariableDeclaration(declaration) = item else {
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
            if guards.writes.contains_key(result) || guards.escaped.contains(result) {
                continue;
            }
            let Some(Expression::AwaitExpression(awaited)) = &declarator.init else {
                continue;
            };
            let Expression::CallExpression(call) = &awaited.argument else {
                continue;
            };
            for next in &body.body[index + 1..] {
                let Statement::ExpressionStatement(expression) = next else {
                    break;
                };
                let Expression::AssignmentExpression(assignment) = &expression.expression else {
                    continue;
                };
                if assignment.operator != oxc_ast::ast::AssignmentOperator::Assign {
                    continue;
                }
                let AssignmentTarget::AssignmentTargetIdentifier(cursor) = &assignment.left else {
                    continue;
                };
                let name = cursor.name.as_str();
                if !cursor_result_member(&assignment.right, result)
                    || name == counter
                    || Some(name) == bound
                {
                    continue;
                }
                let Some(scope) = self.callee_binding_scope(name) else {
                    continue;
                };
                if !self
                    .mutable_cursor_bindings
                    .get(&scope)
                    .is_some_and(|names| names.contains(name))
                    || guards.bindings.contains(name)
                    || guards.writes.get(name).map(Vec::as_slice) != Some(&[assignment.span.start])
                {
                    continue;
                }
                if call
                    .arguments
                    .iter()
                    .filter_map(Argument::as_expression)
                    .any(|arg| cursor_argument_reads(arg, name))
                {
                    self.capped_cursor_calls.insert(call.span.start);
                }
            }
        }
    }
}

fn cursor_plain_references(
    expression: &Expression<'_>,
    include_members: bool,
) -> FxHashSet<String> {
    #[derive(Default)]
    struct References {
        names: FxHashSet<String>,
        include_members: bool,
    }
    impl<'a> Visit<'a> for References {
        fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
            self.names.insert(identifier.name.to_string());
        }
        fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
            if self.include_members {
                walk::walk_static_member_expression(self, member);
            }
        }
    }
    let mut references = References {
        include_members,
        ..Default::default()
    };
    references.visit_expression(expression);
    references.names
}

fn capped_cursor_counter<'a>(
    statement: &'a oxc_ast::ast::ForStatement<'a>,
) -> Option<(&'a str, Option<&'a str>)> {
    let oxc_ast::ast::ForStatementInit::VariableDeclaration(init) = statement.init.as_ref()? else {
        return None;
    };
    if init.kind != VariableDeclarationKind::Let || init.declarations.len() != 1 {
        return None;
    }
    let declaration = &init.declarations[0];
    let counter = binding_identifier_name(&declaration.id)?;
    if !matches!(&declaration.init, Some(Expression::NumericLiteral(value)) if value.value.is_finite())
    {
        return None;
    }
    let Expression::BinaryExpression(test) = statement.test.as_ref()? else {
        return None;
    };
    if !matches!(
        test.operator,
        BinaryOperator::LessThan | BinaryOperator::LessEqualThan
    ) {
        return None;
    }
    if !matches!(&test.left, Expression::Identifier(id) if id.name == counter) {
        return None;
    }
    let bound = match &test.right {
        Expression::Identifier(id) if id.name != counter => Some(id.name.as_str()),
        Expression::NumericLiteral(value) if value.value.is_finite() => None,
        _ => return None,
    };
    let Expression::UpdateExpression(update) = statement.update.as_ref()? else {
        return None;
    };
    if update.operator != oxc_ast::ast::UpdateOperator::Increment {
        return None;
    }
    if !matches!(&update.argument, oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) if id.name == counter)
    {
        return None;
    }
    Some((counter, bound))
}
