//! Record source helper facts in the original AST visit.
use super::{span, Syntax};
use oxc_ast::ast::{Function, FunctionType};

impl Syntax {
    pub(super) fn collect_function_heads(&mut self, value: &Function<'_>) {
        if !self.collect_helper_heads || value.r#type != FunctionType::FunctionDeclaration {
            return;
        }
        let Some(id) = &value.id else {
            return;
        };
        let declaration = span(id.span);
        if let Some(candidate) = super::super::async_returns::function(value) {
            self.async_returns.insert(declaration, candidate);
        }
        if let Some(summary) = super::super::helper_records::function(value) {
            self.record_returns.insert(declaration, summary);
        }
        if let Some(candidate) = super::super::returned_params::function(value) {
            self.returned_params.insert(declaration, candidate);
        }
        let effects = super::super::effects::function(value);
        if effects.iter().any(Option::is_some) {
            self.effect_functions.insert(declaration, effects);
        }
        if let Some(summary) = super::super::helper_heads::function(value) {
            if let Some(reference) = summary.head.reference() {
                self.return_sinks.insert(reference);
            }
            self.helper_returns.insert(declaration, summary);
        }
        if let Some(candidate) = super::super::finite_returns::function(value) {
            if super::super::finite_returns::plain_params(value) {
                self.primitive_functions.insert(declaration);
            }
            self.return_sinks.extend(candidate.returned_locals());
            self.finite_returns.insert(declaration, candidate);
        }
    }
}
