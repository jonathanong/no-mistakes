use super::{EffectIdentities, Targets};
use crate::codebase::postgres::embedded::walk::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, Expression, Function, IdentifierReference,
    UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(in crate::codebase::postgres::embedded::walk::variants::effects) fn contains_identity_writes(
    expression: &Expression<'_>,
) -> bool {
    let mut writes = Writes {
        assigned: None,
        found: false,
    };
    writes.visit_expression(expression);
    writes.found
}

pub(in crate::codebase::postgres::embedded::walk::variants::effects) fn independent_argument(
    visitor: &ScopeVisitor<'_>,
    expression: &Expression<'_>,
) -> bool {
    match unwrap_ts_wrappers(expression) {
        Expression::TaggedTemplateExpression(_) => {
            visitor.builder_identity_for_init(expression).is_some()
        }
        Expression::CallExpression(call) => visitor.fresh_builder_call(call),
        Expression::SequenceExpression(sequence) => {
            copied_scalar(sequence.expressions.last().expect("sequences have a value"))
        }
        _ => copied_scalar(expression),
    }
}

impl EffectIdentities {
    pub(in crate::codebase::postgres::embedded::walk::variants::effects) fn tainted(
        &self,
        expression: &Expression<'_>,
    ) -> bool {
        let mut writes = Writes {
            assigned: Some(&self.assigned),
            found: false,
        };
        writes.visit_expression(expression);
        writes.found
    }
}

struct Writes<'a> {
    assigned: Option<&'a FxHashMap<String, Option<Targets>>>,
    found: bool,
}

impl<'a> Visit<'a> for Writes<'_> {
    fn visit_assignment_expression(&mut self, _: &AssignmentExpression<'a>) {
        self.found = true;
    }
    fn visit_update_expression(&mut self, _: &UpdateExpression<'a>) {
        self.found = true;
    }
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        self.found |= self
            .assigned
            .is_some_and(|assigned| matches!(assigned.get(id.name.as_str()), Some(None)));
    }
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        if !self.found {
            walk::walk_expression(self, expression);
        }
    }
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}

pub(super) fn copied_scalar(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expression),
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
    )
}
