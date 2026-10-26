use super::super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::{ast::Expression, AstKind};
use oxc_span::GetSpan;

impl<'a> ScopeVisitor<'a> {
    pub(in super::super) fn note_plain_member(&mut self, kind: AstKind<'a>) {
        let expression = match kind {
            AstKind::MethodDefinition(method) => {
                self.plain.member_spans.insert(method.value.span.start);
                return;
            }
            AstKind::ObjectProperty(property) => Some(&property.value),
            AstKind::PropertyDefinition(property) => property.value.as_ref(),
            _ => None,
        };
        if let Some(expression) = expression.map(unwrap_ts_wrappers) {
            if matches!(
                expression,
                Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
            ) {
                self.plain.member_spans.insert(expression.span().start);
            }
        }
    }

    pub(super) fn is_plain_member(&self) -> bool {
        self.plain.member_frames.last().copied().unwrap_or(false)
    }
}
