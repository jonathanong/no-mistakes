use super::ScopeVisitor;
use crate::codebase::postgres::embedded::EmbeddedSqlCall;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, IdentifierReference};
use std::num::NonZeroU32;

fn parameter_identity(visitor: &ScopeVisitor<'_>, name: &str) -> Option<NonZeroU32> {
    if visitor.scoped.transaction_parameters.is_empty() {
        return None;
    }
    let identity = visitor.lookup(name)?.value_identity?;
    visitor
        .scoped
        .transaction_parameters
        .contains(&(identity.get() - 1))
        .then_some(identity)
}

pub(super) fn annotate(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    receiver: &Expression<'_>,
    fact: &mut EmbeddedSqlCall,
) -> bool {
    let Expression::Identifier(id) = unwrap_ts_wrappers(receiver) else {
        return false;
    };
    let Some(identity) = parameter_identity(visitor, id.name.as_str()) else {
        return false;
    };
    let optional = call.optional
        || matches!(unwrap_ts_wrappers(&call.callee), Expression::StaticMemberExpression(member) if member.optional)
        || matches!(unwrap_ts_wrappers(&call.callee), Expression::ComputedMemberExpression(member) if member.optional);
    if !optional && !visitor.append_crosses_function(id.name.as_str()) {
        visitor
            .transaction_options
            .caller_transaction_reads
            .borrow_mut()
            .insert(id.span.start);
        fact.caller_transaction_id = Some(identity.get() - 1);
    }
    true
}

impl ScopeVisitor<'_> {
    pub(super) fn note_caller_transaction_escape(&mut self, id: &IdentifierReference<'_>) {
        if let Some(identity) = parameter_identity(self, id.name.as_str()) {
            if !self
                .transaction_options
                .caller_transaction_reads
                .get_mut()
                .remove(&id.span.start)
            {
                self.transaction_options.mutated.insert(identity);
            }
        }
    }

    pub(super) fn finish_transaction_helper_facts(&mut self) {
        self.mark_lock_helper_definitions();
        if self.scoped.transaction_parameters.is_empty()
            || self.transaction_options.mutated.is_empty()
        {
            return;
        }
        for fact in self.calls.iter_mut().chain(
            self.pending_calls
                .iter_mut()
                .map(|pending| &mut pending.call),
        ) {
            if fact.caller_transaction_id.is_some_and(|id| {
                super::transaction::source_identity(id)
                    .is_some_and(|identity| self.transaction_options.mutated.contains(&identity))
            }) {
                fact.caller_transaction_id = None;
            }
        }
    }
}
