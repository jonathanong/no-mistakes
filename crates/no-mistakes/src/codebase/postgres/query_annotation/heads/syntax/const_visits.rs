use super::{head, span, Head, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, VariableDeclaration};
use oxc_span::GetSpan;

impl Syntax {
    pub(super) fn const_declaration(&mut self, value: &VariableDeclaration<'_>) {
        for declaration in &value.declarations {
            if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                if let Some(Expression::ArrowFunctionExpression(arrow)) = self
                    .collect_helper_heads
                    .then(|| declaration.init.as_ref().map(unwrap_ts_wrappers))
                    .flatten()
                {
                    if let Some(summary) = super::super::helper_heads::arrow(arrow) {
                        if let Some(reference) = summary.head.reference() {
                            self.return_sinks.insert(reference);
                        }
                        self.helper_returns.insert(span(binding.span), summary);
                    }
                }
                if self.collect_helper_heads {
                    if let Some(Expression::ArrowFunctionExpression(arrow)) =
                        declaration.init.as_ref().map(unwrap_ts_wrappers)
                    {
                        if let Some(summary) = super::super::helper_records::arrow(arrow) {
                            self.record_returns.insert(span(binding.span), summary);
                        }
                    }
                    if let Some(init) = declaration.init.as_ref() {
                        if let Some(reference) = super::super::helper_heads::callee(init)
                            .or_else(|| super::super::helper_records::append_callee(init))
                        {
                            self.record_initializers
                                .insert(span(binding.span), (reference, span(init.span())));
                        }
                    }
                }
                self.append_declaration(binding, declaration.init.as_ref());
                let head = declaration.init.as_ref().map_or(Head::Unproven, head);
                if head.is_constant() {
                    self.constants.insert(span(binding.span), head);
                    if matches!(
                        declaration.init.as_ref().map(unwrap_ts_wrappers),
                        Some(Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
                    ) {
                        self.string_constants.insert(span(binding.span));
                    }
                }
            }
        }
    }
}
