use super::*;
use oxc_ast::ast::{Expression, IdentifierReference};

impl ImportCollector {
    pub(in super::super::super::super) fn note_metadata_row_expression(
        &mut self,
        expr: &Expression<'_>,
    ) {
        if !self.collect_value_flow {
            return;
        }
        let (object, field, at) = match expr {
            Expression::StaticMemberExpression(member) => (
                &member.object,
                Some(member.property.name.to_string()),
                member.span.start,
            ),
            Expression::ComputedMemberExpression(member) => (
                &member.object,
                super::super::literal_string(&member.expression),
                member.span.start,
            ),
            Expression::UnaryExpression(unary)
                if unary.operator == oxc_syntax::operator::UnaryOperator::LogicalNot =>
            {
                if let Expression::Identifier(identifier) =
                    crate::codebase::ts_source::unwrap_ts_wrappers(&unary.argument)
                {
                    let scope = self
                        .callee_binding_scope(identifier.name.as_str())
                        .unwrap_or(0);
                    if self
                        .metadata_rows
                        .lookups
                        .get(&scope)
                        .is_some_and(|names| names.contains(identifier.name.as_str()))
                    {
                        self.metadata_rows.allowed.insert(identifier.span.start);
                    }
                }
                return;
            }
            _ => return,
        };
        let invalid = self.metadata_rows.writing
            || field.is_none()
            || self.closed_registry_call_callee == Some(at);
        self.note_row_member(object, field.as_deref(), invalid);
    }

    fn note_row_member(&mut self, object: &Expression<'_>, field: Option<&str>, invalid: bool) {
        let root = crate::codebase::ts_source::unwrap_ts_wrappers(object);
        let usage = match root {
            Expression::Identifier(identifier) => {
                let scope = self
                    .callee_binding_scope(identifier.name.as_str())
                    .unwrap_or(0);
                let Some(usage) = self
                    .metadata_rows
                    .named
                    .get_mut(&scope)
                    .and_then(|names| names.get_mut(identifier.name.as_str()))
                else {
                    return;
                };
                self.metadata_rows.allowed.insert(identifier.span.start);
                usage
            }
            Expression::CallExpression(call)
                if super::module_callee(self, &call.callee).is_some() =>
            {
                self.metadata_rows
                    .direct
                    .entry(call.span.start)
                    .or_default()
            }
            Expression::StaticMemberExpression(member) => {
                self.note_row_member(&member.object, None, true);
                return;
            }
            Expression::ComputedMemberExpression(member) => {
                self.note_row_member(&member.object, None, true);
                return;
            }
            _ => return,
        };
        usage.invalid |= invalid;
        if let Some(field) = field {
            usage.fields.insert(field.to_string());
        }
    }

    pub(in super::super::super::super) fn note_metadata_row_identifier(
        &mut self,
        identifier: &IdentifierReference<'_>,
    ) {
        if !self.collect_value_flow || self.metadata_rows.allowed.contains(&identifier.span.start) {
            return;
        }
        let scope = self
            .callee_binding_scope(identifier.name.as_str())
            .unwrap_or(0);
        if let Some(usage) = self
            .metadata_rows
            .named
            .get_mut(&scope)
            .and_then(|names| names.get_mut(identifier.name.as_str()))
        {
            usage.invalid = true;
        }
    }
}
