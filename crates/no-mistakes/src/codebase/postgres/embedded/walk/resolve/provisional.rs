use super::super::super::bindings::{callee_name, member_query_receiver, MEMBER_QUERY_OPT_IN};
use super::super::super::relative::PendingRelativeCall;
use super::super::ScopeVisitor;
use super::executor_call;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(in crate::codebase::postgres::embedded::walk) fn record_executor_call(
    visitor: &mut ScopeVisitor<'_>,
    call: &CallExpression<'_>,
) {
    if visitor.executor_alias_callee(&call.callee) {
        let span = oxc_span::GetSpan::span(&call.callee);
        push_confirmed(
            visitor,
            call,
            visitor.source[span.start as usize..span.end as usize].to_string(),
        );
        visitor.note_imported_alias_call(call);
        return;
    }
    if let Some(receiver) = member_query_receiver(call) {
        if !visitor.query_members {
            return;
        }
        let namespace = visitor.namespace_expression(receiver);
        let broad = visitor.bindings.contains(MEMBER_QUERY_OPT_IN);
        if broad || (visitor.proven_executor(receiver) && (!namespace || visitor.query_binding)) {
            let sequence = push_confirmed(visitor, call, "query".to_string());
            if !broad {
                visitor.note_member_call(sequence, call, receiver);
            }
        } else {
            let owners = visitor.relative_executor_owners(receiver);
            if owners.is_empty() {
                let recovered = executor_call(visitor, call, "query".to_string());
                if looks_like_statement(&recovered) {
                    visitor
                        .unproven_member_query_spans
                        .push((call.span.start, call.span.end));
                }
            }
            let sequence = visitor.next_seq;
            push_pending(visitor, call, "query", owners);
            if visitor.next_seq != sequence {
                visitor.note_member_call(sequence, call, receiver);
            }
        }
        return;
    }
    let member = unwrap_ts_wrappers(&call.callee).is_member_expression();
    if let Some(callee) = callee_name(call, visitor.bindings, visitor.scoped) {
        if !member
            && visitor.proven_executor(&call.callee)
            && !visitor.namespace_expression(&call.callee)
        {
            push_confirmed(visitor, call, callee);
        }
        return;
    }
    if member {
        return;
    }
    let Expression::Identifier(ident) = unwrap_ts_wrappers(&call.callee) else {
        return;
    };
    let owners = visitor.relative_executor_owners(&call.callee);
    push_pending(visitor, call, ident.name.as_str(), owners);
}

fn looks_like_statement(call: &crate::codebase::postgres::embedded::EmbeddedSqlCall) -> bool {
    call.sql_text
        .as_deref()
        .is_some_and(super::super::scope::has_diagnostic_leading_statement)
        || call
            .variants
            .iter()
            .any(|variant| super::super::scope::has_diagnostic_leading_statement(&variant.sql_text))
}

fn push_pending(
    visitor: &mut ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: &str,
    owners: Vec<u32>,
) {
    if owners.is_empty() {
        return;
    }
    let seq = visitor.next_seq;
    visitor.next_seq += 1;
    visitor
        .pending_spans
        .insert(seq, (call.span.start, call.span.end));
    visitor.pending_calls.push(PendingRelativeCall {
        seq,
        owners,
        call: executor_call(visitor, call, callee.to_string()),
    });
    visitor.note_executor_call(call);
}

fn push_confirmed(
    visitor: &mut ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: String,
) -> u32 {
    let seq = visitor.next_seq;
    visitor.next_seq += 1;
    if visitor.track_order {
        visitor.confirmed_order.push(seq);
    }
    visitor.calls.push(executor_call(visitor, call, callee));
    visitor.call_spans.push((call.span.start, call.span.end));
    visitor.note_executor_call(call);
    seq
}
