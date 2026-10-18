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
        return;
    }
    if let Some(receiver) = member_query_receiver(call) {
        if !visitor.query_members {
            return;
        }
        let namespace = visitor.namespace_expression(receiver);
        if visitor.bindings.contains(MEMBER_QUERY_OPT_IN)
            || (visitor.proven_executor(receiver)
                && (!namespace || visitor.namespace_query_valid()))
        {
            push_confirmed(visitor, call, "query".to_string());
        } else {
            let owners = visitor.relative_executor_owners(receiver);
            push_pending(visitor, call, "query", owners);
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
}

fn push_confirmed(visitor: &mut ScopeVisitor<'_>, call: &CallExpression<'_>, callee: String) {
    let seq = visitor.next_seq;
    visitor.next_seq += 1;
    if visitor.track_order {
        visitor.confirmed_order.push(seq);
    }
    visitor.calls.push(executor_call(visitor, call, callee));
    visitor.call_spans.push((call.span.start, call.span.end));
}
