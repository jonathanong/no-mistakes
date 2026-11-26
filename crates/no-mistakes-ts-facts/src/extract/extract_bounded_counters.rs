impl ImportCollector {
    fn constant_counter_loop(&self, statement: &oxc_ast::ast::ForStatement<'_>) -> bool {
        if !self.collect_per_item_calls {
            return false;
        }
        let Some(oxc_ast::ast::ForStatementInit::VariableDeclaration(init)) = &statement.init
        else {
            return false;
        };
        if init.kind != VariableDeclarationKind::Let || init.declarations.len() != 1 {
            return false;
        }
        let declaration = &init.declarations[0];
        let Some(counter) = binding_identifier_name(&declaration.id) else {
            return false;
        };
        let Some(initial) = &declaration.init else {
            return false;
        };
        if self.bounded_number(initial).is_none() {
            return false;
        }
        let Some(Expression::BinaryExpression(test)) = &statement.test else {
            return false;
        };
        if !matches!(&test.left, Expression::Identifier(id) if id.name == counter)
            || self.bounded_number(&test.right).is_none()
        {
            return false;
        }
        let increasing = match test.operator {
            BinaryOperator::LessThan | BinaryOperator::LessEqualThan => true,
            BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan => false,
            _ => return false,
        };
        let step = match statement.update.as_ref() {
            Some(Expression::UpdateExpression(update)) => {
                if !matches!(&update.argument, oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) if id.name == counter)
                    || update.operator
                        != if increasing {
                            oxc_ast::ast::UpdateOperator::Increment
                        } else {
                            oxc_ast::ast::UpdateOperator::Decrement
                        }
                {
                    return false;
                }
                None
            }
            Some(Expression::AssignmentExpression(update)) => {
                if !matches!(&update.left, AssignmentTarget::AssignmentTargetIdentifier(id) if id.name == counter)
                    || update.operator
                        != if increasing {
                            oxc_ast::ast::AssignmentOperator::Addition
                        } else {
                            oxc_ast::ast::AssignmentOperator::Subtraction
                        }
                    || !self
                        .bounded_number(&update.right)
                        .is_some_and(|step| step > 0.0)
                {
                    return false;
                }
                Some(&update.right)
            }
            _ => return false,
        };
        let mut guards = CounterWrites::default();
        guards.visit_statement(&statement.body);
        if guards.opaque || guards.writes.contains(counter) || guards.bindings.contains(counter) {
            return false;
        }
        [Some(initial), Some(&test.right), step]
            .into_iter()
            .flatten()
            .all(|expression| {
                cursor_plain_references(expression, true)
                    .iter()
                    .all(|name| {
                        name != counter
                            && !guards.writes.contains(name)
                            && !guards.bindings.contains(name)
                    })
            })
    }
}
