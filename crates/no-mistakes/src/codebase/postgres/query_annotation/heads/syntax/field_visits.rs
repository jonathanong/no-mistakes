//! Exact non-optional `builder.text` and paired `builder.values` reads.
use super::{span, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(super) fn field<'a>(
    expression: &'a Expression<'a>,
    name: &str,
) -> Option<(&'a str, super::super::Span)> {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    if member.optional || member.property.name != name {
        return None;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    Some((id.name.as_str(), span(id.span)))
}

impl Syntax {
    pub(super) fn field_call(&mut self, call: &CallExpression<'_>) {
        let Some(first) = call.arguments.first().and_then(|arg| arg.as_expression()) else {
            return;
        };
        let Some((name, _)) = field(first, "text") else {
            return;
        };
        let Some(second) = call.arguments.get(1).and_then(|arg| arg.as_expression()) else {
            return;
        };
        if let Some((value_name, reference)) = field(second, "values") {
            if name == value_name {
                self.value_reads.insert(reference, span(call.span));
                if self.collect_helper_heads
                    && !call.optional
                    && (call.arguments.len() == 2
                        || (call.arguments.len() == 3
                            && call.arguments[2].as_expression().is_some()))
                {
                    self.record_result_calls.insert(span(call.span));
                    if call.arguments.len() == 3 {
                        self.record_options_calls.insert(span(call.span));
                    }
                }
            }
        }
    }
}
