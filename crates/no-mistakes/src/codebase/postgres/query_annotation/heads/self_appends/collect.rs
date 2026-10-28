//! Record bounded identity writes during the existing original-AST walk.
use super::{Root, Syntax, Write};
use crate::codebase::postgres::query_annotation::heads::{assignments::Context, syntax, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
fn span(value: oxc_span::Span) -> Span {
    (value.start, value.end)
}
impl Syntax {
    pub fn declaration<'a>(&mut self, value: &'a VariableDeclaration<'a>, context: Context) {
        if value.kind != VariableDeclarationKind::Let
            || context.function.is_none()
            || context.repeated.is_some()
        {
            return;
        }
        for declaration in &value.declarations {
            let BindingPattern::BindingIdentifier(id) = &declaration.id else {
                continue;
            };
            let Some(init) = declaration.init.as_ref() else {
                continue;
            };
            let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(init) else {
                continue;
            };
            let Expression::Identifier(tag) = unwrap_ts_wrappers(&tagged.tag) else {
                continue;
            };
            let head = syntax::head(init);
            let proof = match head {
                syntax::Head::Tag => super::HeadProof::Literal,
                syntax::Head::Text(text) => text.continued,
                _ => super::HeadProof::Unproven,
            };
            self.roots.insert(
                span(id.span),
                Root {
                    tag: span(tag.span),
                    span: span(declaration.span),
                    context,
                    proof,
                },
            );
        }
    }
    pub fn assignment<'a>(
        &mut self,
        value: &'a AssignmentExpression<'a>,
        context: Context,
        discarded: Option<Span>,
    ) {
        if value.operator != AssignmentOperator::Assign
            || discarded != Some(span(value.span))
            || context.repeated.is_some()
        {
            return;
        }
        let AssignmentTarget::AssignmentTargetIdentifier(left) = &value.left else {
            return;
        };
        let mut expression = &value.right;
        let mut links = 0;
        while let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) {
            if links == 8 {
                return;
            }
            let Some(receiver) = super::super::append::prefix_receiver(call) else {
                return;
            };
            links += 1;
            expression = receiver;
        }
        let Expression::Identifier(right) = unwrap_ts_wrappers(expression) else {
            return;
        };
        if links != 0 {
            self.writes.insert(
                span(left.span),
                Write {
                    right: span(right.span),
                    span: span(value.span),
                    context,
                },
            );
        }
    }
    pub fn call<'a>(&mut self, call: &'a CallExpression<'a>) {
        if call.optional
            || call.arguments.len() != 1
            || !matches!(unwrap_ts_wrappers(&call.callee), Expression::Identifier(_))
        {
            return;
        }
        if let Some(Expression::Identifier(id)) =
            call.arguments[0].as_expression().map(unwrap_ts_wrappers)
        {
            self.reads.insert(span(call.span), span(id.span));
        }
    }
}
