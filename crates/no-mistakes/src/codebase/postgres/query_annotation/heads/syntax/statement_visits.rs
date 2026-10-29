//! Expression-statement facts keep the original assignment/discard walk order.
use super::{span, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, ExpressionStatement, ReturnStatement, TSType};
use oxc_ast_visit::walk;
impl Syntax {
    pub(super) fn walk_statement_facts<'a>(&mut self, value: &ExpressionStatement<'a>) {
        if self.collect_helper_heads {
            let effects = super::super::effects::call(&value.expression);
            self.effect_calls.extend(effects.into_iter().flatten());
        }
        let literal = super::super::append::reference(&value.expression);
        if let Some(reference) = literal {
            self.discarded_appends.insert(reference);
        }
        if let Some(reference) = literal
            .is_none()
            .then(|| super::super::append::prefix_reference(&value.expression))
            .flatten()
        {
            self.prefix_discarded_appends.insert(reference);
        }
        let discarded = match unwrap_ts_wrappers(&value.expression) {
            Expression::AssignmentExpression(value) => Some(span(value.span)),
            _ => None,
        };
        let previous = std::mem::replace(&mut self.assignments.discarded_assignment, discarded);
        walk::walk_expression_statement(self, value);
        self.assignments.discarded_assignment = previous;
    }
    pub(super) fn walk_return_facts<'a>(&mut self, value: &ReturnStatement<'a>) {
        if self.collect_helper_heads {
            if let Some(reference) = value
                .argument
                .as_ref()
                .and_then(super::super::append::prefix_reference)
            {
                self.returned_effect_appends.insert(reference);
            }
        }
        walk::walk_return_statement(self, value);
    }
    pub(super) fn walk_type_facts<'a>(&mut self, value: &TSType<'a>) {
        let enabled = self.native_values.replace_enabled(false);
        walk::walk_ts_type(self, value);
        self.native_values.replace_enabled(enabled);
    }
}
