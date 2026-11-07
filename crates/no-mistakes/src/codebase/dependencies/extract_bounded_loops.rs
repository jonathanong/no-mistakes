impl ImportCollector {
    fn record_chunk_calls(&mut self, statement: &oxc_ast::ast::ForStatement<'_>) {
        if !self.collect_per_item_calls || self.inside_per_item(statement.span) {
            return;
        }
        let Some((counter, array, step, step_name)) = chunk_loop(self, statement) else {
            return;
        };
        let mut guards = CursorGuards::default();
        guards.visit_statement(&statement.body);
        if guards.opaque
            || guards.writes.contains_key(counter)
            || guards.bindings.contains(counter)
            || guards.writes.contains_key(array)
            || guards.bindings.contains(array)
            || step_name.is_some_and(|name| {
                guards.writes.contains_key(name) || guards.bindings.contains(name)
            })
        {
            return;
        }
        struct Calls<'b> {
            collector: &'b ImportCollector,
            counter: &'b str,
            array: &'b str,
            step: f64,
            step_name: Option<&'b str>,
            offsets: Vec<u32>,
        }
        impl<'a> Visit<'a> for Calls<'_> {
            fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
                let mut slices = 0;
                if call.arguments.iter().all(|arg| {
                    arg.as_expression().is_some_and(|arg| {
                        chunk_argument(
                            self.collector,
                            arg,
                            self.counter,
                            self.array,
                            (self.step, self.step_name),
                            &mut slices,
                        )
                    })
                }) && slices > 0
                    && !cursor_plain_references(&call.callee, true).contains(self.counter)
                {
                    self.offsets.push(call.span.start);
                }
                walk::walk_call_expression(self, call);
            }
        }
        let mut calls = Calls {
            collector: self,
            counter,
            array,
            step,
            step_name,
            offsets: Vec::new(),
        };
        calls.visit_statement(&statement.body);
        let offsets = calls.offsets;
        for offset in offsets {
            self.record_bounded_batch_call(offset);
        }
    }

    fn inside_per_item(&self, span: oxc_span::Span) -> bool {
        self.per_item_ranges
            .get(&self.current_function_id())
            .into_iter()
            .flatten()
            .any(|range| {
                range.span.start <= span.start
                    && span.end <= range.span.end
                    && range
                        .guard
                        .as_ref()
                        .is_none_or(|guard| !self.bounded_domains.guard_valid(guard))
            })
    }
}

fn chunk_loop<'a>(
    collector: &ImportCollector,
    statement: &'a oxc_ast::ast::ForStatement<'a>,
) -> Option<(&'a str, &'a str, f64, Option<&'a str>)> {
    let oxc_ast::ast::ForStatementInit::VariableDeclaration(init) = statement.init.as_ref()? else {
        return None;
    };
    if init.kind != VariableDeclarationKind::Let || init.declarations.len() != 1 {
        return None;
    }
    let declaration = &init.declarations[0];
    if !matches!(&declaration.init, Some(Expression::NumericLiteral(value)) if value.value == 0.0) {
        return None;
    }
    let counter = binding_identifier_name(&declaration.id)?;
    let Expression::BinaryExpression(test) = statement.test.as_ref()? else {
        return None;
    };
    if test.operator != BinaryOperator::LessThan
        || !matches!(&test.left, Expression::Identifier(id) if id.name == counter)
    {
        return None;
    }
    let Expression::StaticMemberExpression(length) = &test.right else {
        return None;
    };
    if length.property.name != "length" {
        return None;
    }
    let Expression::Identifier(array) = &length.object else {
        return None;
    };
    let Expression::AssignmentExpression(update) = statement.update.as_ref()? else {
        return None;
    };
    if update.operator != oxc_ast::ast::AssignmentOperator::Addition
        || !matches!(&update.left, AssignmentTarget::AssignmentTargetIdentifier(id) if id.name == counter)
    {
        return None;
    }
    let step = collector.bounded_number(&update.right)?;
    let step_name = match &update.right {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    };
    (step > 1.0 && step.fract() == 0.0).then_some((counter, array.name.as_str(), step, step_name))
}
