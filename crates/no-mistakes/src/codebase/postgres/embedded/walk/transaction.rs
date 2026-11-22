use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{bindings::member_query_receiver, EmbeddedSqlCall};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{ArrayExpressionElement, CallExpression, Expression};
mod boundary;
pub(super) mod options;
pub(super) use boundary::end_boundary;

pub(super) fn annotate(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    fact: &mut EmbeddedSqlCall,
) {
    fact.execution_awaited = visitor.awaited_lock_helper_calls.contains(&call.span.start);
    fact.lock_scope_analysis = visitor.executor_options.track_transaction_helpers;
    fact.track_share_lock_writes = visitor.executor_options.track_share_lock_writes;
    fact.track_redundant_prelocks = visitor.executor_options.track_redundant_prelocks;
    if fact.track_redundant_prelocks {
        fact.next_discarded_query = visitor
            .adjacent_discarded_queries
            .get(&call.span.start)
            .copied();
    }
    let owner = visitor
        .member_query_proof
        .function_ids
        .last()
        .copied()
        .unwrap_or(0);
    if visitor.executor_options.track_transaction_helpers {
        fact.execution_scope = owner;
        fact.execution_range = (call.span.start, call.span.end);
    }
    fact.execution_unconditional = visitor.control_depth == 0
        && visitor.loop_depth == 0
        && !visitor.exited_functions.contains(&owner);
    let routed = options::annotate(visitor, call, fact);
    if routed.is_none() {
        let receiver = member_query_receiver(call).unwrap_or(&call.callee);
        if !super::caller_transactions::annotate(visitor, call, receiver, fact) {
            annotate_receiver(visitor, receiver, call.span.start, fact);
        }
    }
    super::stable_binds::annotate(visitor, call, fact);
    if routed == Some(false) {
        return;
    }
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
                                    .and_then(|value| identity(visitor, value))
                                    .filter(|identity| {
                                        fact.caller_transaction_id.is_none()
                                            || stable_parameter_identity(visitor, identity)
                                    }),
                            })
                            .collect(),
                    )
                }
                _ => None,
            })
            .unwrap_or_default();
    if fact.track_redundant_prelocks {
        fact.stable_prelock_bindings = fact.bound_parameters.iter().all(|parameter| {
            parameter
                .as_deref()
                .is_some_and(|identity| stable_parameter_identity(visitor, identity))
        });
    }
}

pub(super) fn stable_parameter_identity(visitor: &ScopeVisitor<'_>, identity: &str) -> bool {
    identity.starts_with("literal:")
        || identity
            .strip_prefix("binding:")
            .and_then(|position| position.parse::<u32>().ok())
            .is_some_and(|position| visitor.stable_primitives.contains(&position))
}

pub(super) fn annotate_receiver(
    visitor: &ScopeVisitor<'_>,
    receiver: &Expression<'_>,
    position: u32,
    fact: &mut EmbeddedSqlCall,
) {
    if visitor.executor_options.track_transaction_helpers
        && options::annotate_alias(visitor, receiver, fact)
    {
        return;
    }
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
                !visitor.scoped.transaction_parameters.contains(&span.start)
                    && binding.as_ref().and_then(|value| value.value_identity)
                        == source_identity(span.start)
            });
        if let Some(span) = transaction {
            fact.transaction_id = Some(span.start);
            fact.transaction_line = Some(byte_offset_to_line(visitor.source, span.start as usize));
            fact.executor_transaction_known = !visitor.append_crosses_function(name)
                && (!visitor.executor_options.track_transaction_helpers
                    || source_identity(span.start).is_some_and(|identity| {
                        !visitor.transaction_options.mutated.contains(&identity)
                    }));
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

pub(super) fn source_identity(start: u32) -> Option<std::num::NonZeroU32> {
    start.checked_add(1).and_then(std::num::NonZeroU32::new)
}
