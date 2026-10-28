//! Primitive append syntax shares the original head walk and assignment contexts.
use super::{head, span, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;

fn primitive(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expression),
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
    )
}
impl Syntax {
    pub(super) fn string_declaration(&mut self, value: &VariableDeclaration<'_>) {
        if !self.collect_string_prefix || value.kind != VariableDeclarationKind::Let {
            return;
        }
        for declaration in &value.declarations {
            let BindingPattern::BindingIdentifier(id) = &declaration.id else {
                continue;
            };
            if let Some(init) = declaration.init.as_ref().filter(|init| primitive(init)) {
                self.string_heads.insert(span(id.span), head(init));
                self.contexts.insert(span(id.span), self.current_function);
            }
        }
    }
    pub(super) fn string_assignment(&mut self, value: &AssignmentExpression<'_>) {
        self.assignments.assignment(value);
        if self.collect_string_prefix
            && value.operator == AssignmentOperator::Addition
            && self.assignments.discarded_assignment == Some(span(value.span))
            && primitive(&value.right)
        {
            if let AssignmentTarget::AssignmentTargetIdentifier(id) = &value.left {
                self.string_appends.insert(span(id.span));
            }
        }
    }
}
