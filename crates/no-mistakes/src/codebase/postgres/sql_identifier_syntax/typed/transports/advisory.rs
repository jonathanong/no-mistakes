use super::*;

impl Typed {
    pub(super) fn advisory_factory_source(&self, expression: &Expression<'_>) -> Option<Span> {
        let expression = unwrap_ts_wrappers(expression);
        let expression = if let Expression::AwaitExpression(value) = expression {
            unwrap_ts_wrappers(&value.argument)
        } else {
            expression
        };
        let Expression::CallExpression(call) = expression else {
            return None;
        };
        match unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(id) => {
                let at = span(id.span);
                self.bindings
                    .get(&at)
                    .is_some_and(|id| self.advisory_factory_defaults.contains(id))
                    .then_some(at)
            }
            Expression::StaticMemberExpression(member) => {
                self.advisory_member(&member.object, member.property.name.as_str())
            }
            Expression::ComputedMemberExpression(member) => {
                let Expression::StringLiteral(key) = unwrap_ts_wrappers(&member.expression) else {
                    return None;
                };
                self.advisory_member(&member.object, crate::ast::js_literal(key).as_ref())
            }
            _ => None,
        }
    }

    fn advisory_member(&self, object: &Expression<'_>, name: &str) -> Option<Span> {
        let Expression::Identifier(id) = unwrap_ts_wrappers(object) else {
            return None;
        };
        let at = span(id.span);
        self.bindings
            .get(&at)
            .and_then(|id| self.advisory_factory_members.get(id))
            .is_some_and(|members| members.contains(name))
            .then_some(at)
    }
}
