use super::*;

impl ImportCollector {
    /// A narrow row-origin proof, not a mutation evaluator: no intervening
    /// statement or second interpolation can execute before this member read.
    pub(in super::super) fn note_row_literal_overwrite(
        &mut self,
        previous: Option<&Statement<'_>>,
        current: &Statement<'_>,
    ) {
        if !self.collect_value_flow || self.value_flow.iteration_bindings.is_empty() {
            return;
        }
        let Some(Statement::ExpressionStatement(previous)) = previous else {
            return;
        };
        let Expression::AssignmentExpression(assignment) = &previous.expression else {
            return;
        };
        if !assignment.operator.is_assign()
            || !matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(&assignment.right),
                Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::BigIntLiteral(_)
                    | Expression::NullLiteral(_)
            )
        {
            return;
        }
        let AssignmentTarget::StaticMemberExpression(target) = &assignment.left else {
            return;
        };
        let Expression::Identifier(root) = &target.object else {
            return;
        };
        let Statement::ExpressionStatement(current) = current else {
            return;
        };
        let Expression::CallExpression(call) = &current.expression else {
            return;
        };
        if !matches!(call.callee, Expression::Identifier(_)) || call.arguments.len() != 1 {
            return;
        }
        let Some(Expression::TemplateLiteral(template)) = call.arguments[0].as_expression() else {
            return;
        };
        if template.expressions.len() != 1 {
            return;
        }
        let Expression::StaticMemberExpression(read) = &template.expressions[0] else {
            return;
        };
        let Expression::Identifier(reader) = &read.object else {
            return;
        };
        if reader.name != root.name || read.property.name != target.property.name {
            return;
        }
        let Some(scope) = self.callee_binding_scope(root.name.as_str()) else {
            return;
        };
        self.value_flow.row_literal_overwrites.push((
            call.span.start,
            call.span.end,
            scope,
            root.name.to_string(),
            target.property.name.to_string(),
        ));
    }
}
