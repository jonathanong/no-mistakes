use super::*;

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn poison_closed_iterable(
        &mut self,
        value: &Expression<'_>,
    ) {
        if expression(self,value,8).is_some_and(|arms| arms.iter().all(|arm|matches!(&arm.node,Node::Array(items) if items.iter().all(|item|matches!(item,Node::Text(_)))))) { return; }
        self.poison_closed_result(value);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn poison_closed_constructor(
        &mut self,
        created: &NewExpression<'_>,
    ) {
        let safe_set = self.set_intrinsic_safe
            && self.lookup("Set").is_none()
            && matches!(unwrap_ts_wrappers(&created.callee),Expression::Identifier(id) if id.name=="Set");
        for argument in &created.arguments {
            let value = match argument {
                Argument::SpreadElement(spread) => Some(&spread.argument),
                _ => argument.as_expression(),
            };
            let Some(value) = value else { continue };
            if safe_set && expression(self,value,8).is_some_and(|arms| arms.iter().all(|arm|matches!(&arm.node,Node::Array(items) if items.iter().all(|item|matches!(item,Node::Text(_)))))) { continue; }
            self.poison_closed_result(value);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn poison_closed_arguments(
        &mut self,
        call: &CallExpression<'_>,
    ) {
        if self.head_calls_only {
            return;
        }
        let allowlist = if !call.optional
            && !call
                .arguments
                .iter()
                .any(|arg| matches!(arg, Argument::SpreadElement(_)))
        {
            match unwrap_ts_wrappers(&call.callee) {
                Expression::Identifier(id) if !self.shadowed_locally(id.name.as_str()) => self
                    .literal_objects
                    .closed
                    .assertions
                    .get(id.name.as_str())
                    .map(|entry| entry.allowlist_argument),
                _ => None,
            }
        } else {
            None
        };
        for (index, arg) in call.arguments.iter().enumerate() {
            if allowlist == Some(index) {
                continue;
            }
            let value = match arg {
                Argument::SpreadElement(spread) => Some(&spread.argument),
                _ => arg.as_expression(),
            };
            if let Some(value) = value {
                self.poison_closed_result(value);
            }
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn poison_closed_result(
        &mut self,
        value: &Expression<'_>,
    ) {
        if self.head_calls_only {
            return;
        }
        let mut pending = vec![value];
        let mut roots = Vec::new();
        while let Some(value) = pending.pop() {
            let value = unwrap_ts_wrappers(value);
            if let Some(arms) = expression(self, value, 8) {
                roots.extend(
                    arms.into_iter()
                        .filter(|arm| !matches!(arm.node, Node::Text(_)))
                        .filter_map(|arm| arm.root),
                );
                continue;
            }
            match value {
                Expression::ObjectExpression(object) => {
                    pending.extend(object.properties.iter().map(|property| match property {
                        ObjectPropertyKind::ObjectProperty(property) => &property.value,
                        ObjectPropertyKind::SpreadProperty(spread) => &spread.argument,
                    }))
                }
                Expression::ArrayExpression(array) => {
                    pending.extend(array.elements.iter().filter_map(|element| match element {
                        ArrayExpressionElement::SpreadElement(spread) => Some(&spread.argument),
                        _ => element.as_expression(),
                    }))
                }
                Expression::ConditionalExpression(branch) => {
                    pending.extend([&branch.consequent, &branch.alternate])
                }
                Expression::LogicalExpression(branch) => {
                    pending.extend([&branch.left, &branch.right])
                }
                Expression::SequenceExpression(sequence) => {
                    pending.extend(sequence.expressions.last())
                }
                Expression::StaticMemberExpression(member) => pending.push(&member.object),
                Expression::ComputedMemberExpression(member) => pending.push(&member.object),
                Expression::AwaitExpression(awaited) => pending.push(&awaited.argument),
                Expression::YieldExpression(yielded) => pending.extend(yielded.argument.as_ref()),
                _ => {}
            }
        }
        self.poison_closed_roots(roots);
    }
}
