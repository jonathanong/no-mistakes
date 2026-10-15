fn visit_for_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForStatement<'a>,
) {
    collector.record_per_item_range(
        statement.body.span(),
        collector.current_function_id(),
        "for",
    );
    let pushed = collector.push_lexical_scope();
    if let Some(init) = &statement.init {
        collector.visit_for_statement_init(init);
    }
    // Setup executes once per statement; preserve any enclosing repeat guard.
    collector.repeat_depth += 1;
    if let Some(test) = &statement.test {
        collector.visit_expression(test);
    }
    if let Some(update) = &statement.update {
        with_conditional_execution(collector, true, |collector| {
            collector.visit_expression(update)
        });
    }
    let conditional = statement
        .test
        .as_ref()
        .is_some_and(|test| literal_truthiness(test) != Some(true));
    with_conditional_execution(collector, conditional, |collector| {
        with_conditional_receiver_write(collector, |collector| {
            with_replacement_region(collector, statement.body.span(), false, true, |collector| {
                collector.visit_statement(&statement.body)
            });
        });
    });
    collector.repeat_depth -= 1;
    collector.pop_lexical_scope(pushed);
}

fn visit_for_in_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForInStatement<'a>,
) {
    collector.record_per_item_range(
        statement.body.span(),
        collector.current_function_id(),
        "for-in",
    );
    let pushed = collector.push_lexical_scope();
    invalidate_for_statement_assignment_target(
        collector,
        &statement.left,
        statement.right.span().end,
    );
    // In sloppy scripts Annex B evaluates `var key = init` before enumerating
    // keys. Keep the target and any destructuring defaults per iteration.
    let initializer = if collector.sloppy_script {
        match &statement.left {
            ForStatementLeft::VariableDeclaration(declaration)
                if declaration.kind == VariableDeclarationKind::Var
                    && declaration.declarations.len() == 1 =>
            {
                declaration
                    .declarations
                    .first()
                    .filter(|declarator| binding_identifier_name(&declarator.id).is_some())
                    .and_then(|declarator| declarator.init.as_ref())
                    .map(|init| {
                        (
                            init.span(),
                            collector.conditional_execution,
                            collector.receiver_write_is_conditional,
                        )
                    })
            }
            _ => None,
        }
    } else {
        None
    };
    let previous_initializer =
        std::mem::replace(&mut collector.annex_b_for_in_initializer, initializer);
    with_repeat_execution(collector, &statement.left, |collector, left| {
        with_conditional_execution(collector, true, |collector| {
            collector.visit_for_statement_left(left)
        });
    });
    collector.annex_b_for_in_initializer = previous_initializer;
    collector.visit_expression(&statement.right);
    with_repeat_execution(collector, &statement.body, |collector, body| {
        with_conditional_execution(collector, true, |collector| {
            with_replacement_region(collector, body.span(), false, true, |collector| {
                collector.visit_statement(body)
            });
        });
    });
    collector.pop_lexical_scope(pushed);
}

fn visit_for_of_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForOfStatement<'a>,
) {
    collector.record_per_item_range(
        statement.body.span(),
        collector.current_function_id(),
        "for-of",
    );
    let pushed = collector.push_lexical_scope();
    invalidate_for_statement_assignment_target(
        collector,
        &statement.left,
        statement.right.span().end,
    );
    with_repeat_execution(collector, &statement.left, |collector, left| {
        with_conditional_execution(collector, true, |collector| {
            collector.visit_for_statement_left(left)
        });
    });
    collector.visit_expression(&statement.right);
    let simple_left = matches!(&statement.left, ForStatementLeft::VariableDeclaration(declaration)
        if declaration.declarations.len() == 1
            && matches!(&declaration.declarations[0].id, BindingPattern::BindingIdentifier(_)));
    let may_repeat = !matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&statement.right),
        Expression::ArrayExpression(array) if array.elements.len() <= 1
            && array.elements.iter().all(|element| !matches!(element, oxc_ast::ast::ArrayExpressionElement::SpreadElement(_))));
    let repeated = simple_left
        && !statement.r#await
        && matches!(
            crate::codebase::ts_source::unwrap_ts_wrappers(&statement.right),
            Expression::ArrayExpression(array) if array.elements.len() >= 2
                && array.elements.iter().all(|element| !matches!(element, oxc_ast::ast::ArrayExpressionElement::SpreadElement(_)))
        );
    // A straight-line assignment executes on the first iteration of this
    // literal, nonempty loop. The receiver replacement region remains unsafe
    // for other purposes because assigning a mutable local invalidates it.
    let guaranteed_assignment = if repeated
        && !collector.conditional_execution
        && collector.current_function_id().is_none()
        && matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&statement.right),
            Expression::ArrayExpression(array) if array.elements.iter().all(|element| {
                matches!(element.as_expression(),
                    Some(Expression::NumericLiteral(_)
                        | Expression::StringLiteral(_)
                        | Expression::BooleanLiteral(_)))
            }))
    {
        match &statement.body {
            Statement::BlockStatement(block) => match block.body.as_slice() {
                [Statement::ExpressionStatement(expression)] => {
                    match crate::codebase::ts_source::unwrap_ts_wrappers(&expression.expression) {
                        Expression::AssignmentExpression(assignment)
                            if assignment.operator == oxc_syntax::operator::AssignmentOperator::Assign
                                && matches!(assignment.left, AssignmentTarget::AssignmentTargetIdentifier(_))
                                && matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&assignment.right),
                                    Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)) =>
                        {
                            Some(assignment.span.end)
                        }
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => None,
        }
    } else { None };
    with_repeat_execution(collector, &statement.body, |collector, body| {
        with_conditional_execution(collector, true, |collector| {
            with_replacement_region(collector, body.span(), repeated, may_repeat, |collector| {
                collector.visit_statement(body);
            });
        });
    });
    if let Some(completed_at) = guaranteed_assignment {
        for write in &mut collector.value_flow.callable_assignments {
            if write.completed_at == completed_at && write.callable.is_some() {
                write.guaranteed = true;
            }
        }
    }
    collector.pop_lexical_scope(pushed);
}

fn invalidate_for_statement_assignment_target(
    collector: &mut ImportCollector,
    left: &ForStatementLeft<'_>,
    completed_at: u32,
) {
    let Some(target) = left.as_assignment_target() else {
        return;
    };
    collector.record_callable_assignment(target, None, false, completed_at);
    collector.record_replacement_target(target, ValueExpression::Unknown, completed_at);
    for (name, path, _) in extract_value_flow::write_targets::targets(target) {
        collector.record_reassigned_callable_alias_path(
            &name,
            target.span().start,
            Some(&path),
            completed_at,
        );
    }
}
