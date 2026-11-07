use crate::codebase::dependencies::extract::{CallableId, ValueExpression};
use oxc_ast::ast::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedIdentifierReturn {
    pub(crate) owner: CallableId,
    pub(crate) prefix: ValueExpression,
}

impl super::super::super::ImportCollector {
    pub(in super::super::super) fn note_generated_identifier_return(
        &mut self,
        expr: &Expression<'_>,
    ) {
        let Some(owner) = self.current_function_id() else {
            return;
        };
        let Expression::TemplateLiteral(template) =
            crate::codebase::ts_source::unwrap_ts_wrappers(expr)
        else {
            return;
        };
        if template.expressions.len() != 2
            || !crate::ast::template_quasi_text(&template.quasis[0]).is_empty()
            || crate::ast::template_quasi_text(&template.quasis[1]) != "_"
            || !crate::ast::template_quasi_text(&template.quasis[2]).is_empty()
            || !super::names::stripped_uuid(&template.expressions[1], &|uuid| {
                self.trusted_uuid_call(uuid)
            })
        {
            return;
        }
        self.value_flow
            .generated_identifier_returns
            .push(GeneratedIdentifierReturn {
                owner,
                prefix: self.value_expression(&template.expressions[0]),
            });
    }
}

#[cfg(test)]
#[path = "closed_identifier_generated_tests.rs"]
mod tests;
