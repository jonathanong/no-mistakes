//! Const initializer models are syntax facts from the existing parsed program.
use super::{lower, span, FileModels, LocalModel};
use oxc_ast::ast::{BindingPattern, VariableDeclaration, VariableDeclarationKind};
use oxc_span::GetSpan;

pub(super) fn collect(
    models: &mut FileModels,
    declaration: &VariableDeclaration<'_>,
    ctx: &lower::Context<'_>,
) {
    if declaration.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &declaration.declarations {
        let (BindingPattern::BindingIdentifier(binding), Some(initializer)) =
            (&declarator.id, &declarator.init)
        else {
            continue;
        };
        let Some(id) = ctx.bindings.id(binding.span) else {
            continue;
        };
        let callable = match crate::codebase::ts_source::unwrap_ts_wrappers(initializer) {
            oxc_ast::ast::Expression::ArrowFunctionExpression(value) => Some(value.span.start),
            oxc_ast::ast::Expression::FunctionExpression(value) => Some(value.span.start),
            _ => None,
        };
        if let Some(callable) = callable {
            models.signature_binding(id, super::CallableId(callable));
        }
        let local = LocalModel {
            declaration: span(declarator.span),
            initializer: span(initializer.span()),
            lowered: lower::expression(initializer, ctx),
        };
        if models.locals.insert(id, local).is_some() {
            models.locals.remove(&id);
        }
    }
}
