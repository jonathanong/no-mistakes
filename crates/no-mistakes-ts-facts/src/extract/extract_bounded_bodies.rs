impl ImportCollector {
    fn closed_body(
        &self,
        collection: &Expression<'_>,
        body: &Statement<'_>,
        keys: bool,
    ) -> Option<BoundedGuard> {
        if !self.collect_per_item_calls {
            return None;
        }
        let key_guard = if keys {
            if let Expression::Identifier(id) = collection {
                self.bounded_binding_guard(id.name.as_str(), true)
            } else {
                None
            }
        } else {
            None
        };
        let guard = key_guard.or_else(|| self.closed_iteration(collection))?;
        let mut writes = CursorGuards::default();
        writes.visit_statement(body);
        bounded_body_preserves(&guard, &writes).then_some(guard)
    }

    fn closed_callback(
        &self,
        call: &CallExpression<'_>,
        callback: &Expression<'_>,
    ) -> Option<BoundedGuard> {
        if !self.collect_per_item_calls || self.inside_per_item(call.span) {
            return None;
        }
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return None;
        };
        let guard = self.closed_iteration(&member.object)?;
        if !bounded_callback_hides_source(callback) {
            return None;
        }
        let mut writes = CursorGuards::default();
        match callback {
            Expression::ArrowFunctionExpression(function) => {
                if let Some(body) = crate::ast::arrow_function_body(&function.body) {
                    writes.visit_function_body(body);
                } else if let Some(expression) = function.body.as_expression() {
                    writes.visit_expression(expression);
                }
            }
            Expression::FunctionExpression(function) => {
                writes.visit_function_body(function.body.as_ref()?);
            }
            _ => return None,
        }
        bounded_body_preserves(&guard, &writes).then_some(guard)
    }

    fn record_iteration_range(
        &mut self,
        span: oxc_span::Span,
        caller: CallableId,
        construct: &str,
        guard: Option<BoundedGuard>,
    ) {
        if let Some(guard) = guard {
            self.record_closed_per_item_range(span, Some(caller), construct, guard);
        } else {
            self.record_per_item_range(span, Some(caller), construct);
        }
    }
}

fn bounded_body_preserves(guard: &BoundedGuard, writes: &CursorGuards) -> bool {
    !writes.opaque
        && guard.iter().all(|(_, root)| {
            !writes.writes.contains_key(root)
                && !writes.escaped.contains(root)
                && !writes.bindings.contains(root)
        })
}

fn bounded_callback_hides_source(callback: &Expression<'_>) -> bool {
    let params = match callback {
        Expression::ArrowFunctionExpression(function) => &function.params,
        Expression::FunctionExpression(function) => &function.params,
        _ => return false,
    };
    params.items.len() <= 2
        && params.rest.is_none()
        && !cursor_plain_references(callback, true).contains("arguments")
}
