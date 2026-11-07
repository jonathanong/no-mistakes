use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{bindings::member_query_receiver, EmbeddedSqlCall};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{ArrayExpressionElement, CallExpression, Expression};
mod options;

pub(super) fn annotate(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    fact: &mut EmbeddedSqlCall,
) {
    let owner = visitor
        .member_query_proof
        .function_ids
        .last()
        .copied()
        .unwrap_or(0);
    fact.execution_unconditional = visitor.control_depth == 0
        && visitor.loop_depth == 0
        && !visitor.exited_functions.contains(&owner);
    let override_receiver =
        options::receiver(call, &visitor.executor_options.executor_option_properties);
    let receiver = match override_receiver {
        Some(Some(receiver)) => receiver,
        Some(None) => {
            fact.executor_transaction_known = false;
            return;
        }
        None => member_query_receiver(call).unwrap_or(&call.callee),
    };
    annotate_receiver(visitor, receiver, call.span.start, fact);
    fact.bound_parameters =
        call.arguments
            .get(1)
            .and_then(|argument| argument.as_expression())
            .and_then(|expression| match unwrap_ts_wrappers(expression) {
                Expression::ArrayExpression(array)
                    if !array.elements.iter().any(|element| {
                        matches!(element, ArrayExpressionElement::SpreadElement(_))
                    }) =>
                {
                    Some(
                        array
                            .elements
                            .iter()
                            .map(|element| match element {
                                ArrayExpressionElement::SpreadElement(_)
                                | ArrayExpressionElement::Elision(_) => None,
                                _ => element
                                    .as_expression()
                                    .and_then(|value| identity(visitor, value)),
                            })
                            .collect(),
                    )
                }
                _ => None,
            })
            .unwrap_or_default();
}

pub(super) fn annotate_receiver(
    visitor: &ScopeVisitor<'_>,
    receiver: &Expression<'_>,
    position: u32,
    fact: &mut EmbeddedSqlCall,
) {
    if let Expression::Identifier(id) = unwrap_ts_wrappers(receiver) {
        let name = id.name.as_str();
        let binding = visitor.lookup(name);
        let transaction = visitor
            .scoped
            .transactions
            .get(name)
            .and_then(|spans| {
                spans
                    .iter()
                    .filter(|span| span.start <= position && position < span.end)
                    .max_by_key(|span| span.start)
            })
            .filter(|span| {
                binding.as_ref().and_then(|value| value.value_identity)
                    == source_identity(span.start)
            });
        if let Some(span) = transaction {
            fact.transaction_id = Some(span.start);
            fact.transaction_line = Some(byte_offset_to_line(visitor.source, span.start as usize));
            fact.executor_transaction_known = !visitor.append_crosses_function(name);
        } else {
            fact.executor_transaction_known = binding.is_none()
                && (visitor.bindings.contains(name)
                    || visitor.imported_namespace_names.contains_key(name));
        }
    }
}

pub(super) fn identity(visitor: &ScopeVisitor<'_>, expression: &Expression<'_>) -> Option<String> {
    if let Expression::Identifier(id) = unwrap_ts_wrappers(expression) {
        let binding = visitor.lookup(id.name.as_str())?;
        let identity = binding.value_identity?;
        return Some(
            visitor
                .scalar_literals
                .get(&identity)
                .cloned()
                .unwrap_or_else(|| format!("binding:{}", identity.get() - 1)),
        );
    }
    literal_identity(expression)
}

pub(super) fn literal_identity(expression: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(value) => Some(format!(
            "literal:'{}'",
            crate::ast::js_literal(value).replace('\'', "''")
        )),
        Expression::NumericLiteral(value) => Some(format!("literal:{}", value.value)),
        Expression::BooleanLiteral(value) => Some(format!("literal:{}", value.value)),
        Expression::NullLiteral(_) => Some("literal:null".into()),
        _ => None,
    }
}

pub(super) fn end_boundary(
    visitor: &mut ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    before: usize,
) {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return;
    };
    let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) else {
        return;
    };
    let method_end = matches!(member.property.name.as_str(), "commit" | "rollback");
    let sql_end = visitor.calls[before..].iter().any(|fact| {
        fact.sql_text.as_deref().is_some_and(|sql| {
            matches!(
                sql.trim()
                    .trim_end_matches(';')
                    .trim()
                    .to_ascii_uppercase()
                    .as_str(),
                "COMMIT" | "ROLLBACK" | "END" | "ABORT"
            )
        })
    });
    if (method_end || sql_end) && visitor.scoped.transactions.contains_key(id.name.as_str()) {
        for scope in visitor.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(id.name.as_str()) {
                binding.value_identity = None;
                break;
            }
        }
    }
}

pub(super) fn source_identity(start: u32) -> Option<std::num::NonZeroU32> {
    start.checked_add(1).and_then(std::num::NonZeroU32::new)
}
