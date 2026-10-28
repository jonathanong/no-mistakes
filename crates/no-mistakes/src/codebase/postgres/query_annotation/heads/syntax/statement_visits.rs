//! Expression-statement facts keep the original assignment/discard walk order.
use super::{span, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, ExpressionStatement};
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
}
