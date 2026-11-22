use super::{options, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(in crate::codebase::postgres::embedded::walk) fn end_boundary(
    visitor: &mut ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    before: usize,
) {
    if visitor.executor_options.track_transaction_helpers {
        let owner = visitor
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        if visitor.exited_functions.contains(&owner) {
            return;
        }
    }
    let (receiver, method_end) = match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member) => (
            &member.object,
            matches!(member.property.name.as_str(), "commit" | "rollback"),
        ),
        Expression::Identifier(_) if visitor.executor_options.track_transaction_helpers => {
            (&call.callee, false)
        }
        _ => return,
    };
    let Expression::Identifier(id) = unwrap_ts_wrappers(receiver) else {
        return;
    };
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
    super::super::transaction_flow::note_boundary(visitor, call, method_end || sql_end);
    if (method_end || sql_end)
        && visitor.executor_options.track_transaction_helpers
        && options::end_alias(visitor, id, call)
    {
        return;
    }
    if (method_end || sql_end) && visitor.scoped.transactions.contains_key(id.name.as_str()) {
        let identity = visitor
            .scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(id.name.as_str()))
            .and_then(|binding| binding.value_identity.take());
        super::super::transaction_flow::complete_boundary(visitor, call, identity);
    }
}
