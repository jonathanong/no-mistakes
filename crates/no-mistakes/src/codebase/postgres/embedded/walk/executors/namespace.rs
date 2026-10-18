use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use std::collections::{BTreeMap, HashSet};

#[derive(Default)]
pub(crate) struct MemberQueryProof {
    namespace_invalidated: bool,
    receiver_invalidated: bool,
    calls: BTreeMap<u32, (u32, u32, bool)>,
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn namespace_expression(
        &self,
        expression: &Expression<'_>,
    ) -> bool {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                match self
                    .scopes
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, scope)| scope.contains_key(name))
                {
                    Some((index, scope)) => {
                        scope[name].executor_alias == Some(true)
                            && self.namespace_proofs[index].contains(name)
                    }
                    None => self.imported_namespace_names.contains(name),
                }
            }
            Expression::CallExpression(call) => self.configured_require(call),
            Expression::ConditionalExpression(conditional) => {
                self.namespace_expression(&conditional.consequent)
                    || self.namespace_expression(&conditional.alternate)
            }
            _ => false,
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn namespace_query_valid(&self) -> bool {
        self.query_binding && !self.member_query_proof.namespace_invalidated
    }

    pub(in crate::codebase::postgres::embedded::walk) fn receiver_query_valid(&self) -> bool {
        !self.member_query_proof.receiver_invalidated
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_member_call(
        &mut self,
        sequence: u32,
        call: &CallExpression<'_>,
        namespace: bool,
    ) {
        self.member_query_proof
            .calls
            .insert(sequence, (call.span.start, call.span.end, namespace));
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_query_receiver(
        &mut self,
        receiver: &Expression<'_>,
    ) {
        if self.namespace_expression(receiver) {
            self.member_query_proof.namespace_invalidated = true;
            for (scope, proofs) in self.scopes.iter_mut().zip(&self.namespace_proofs) {
                for (name, binding) in scope {
                    if proofs.contains(name) {
                        binding.executor_alias = None;
                    }
                }
            }
        } else if self.proven_executor(receiver) || self.relative_executor_proof(receiver).is_some()
        {
            self.member_query_proof.receiver_invalidated = true;
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_namespace_arguments(
        &mut self,
        call: &CallExpression<'_>,
    ) {
        for argument in &call.arguments {
            if let Some(expression) = argument.as_expression() {
                if self.namespace_expression(expression)
                    || self.proven_executor(expression)
                    || self.relative_executor_proof(expression).is_some()
                {
                    self.revoke_query_receiver(expression);
                }
            }
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn drop_invalidated_member_calls(&mut self) {
        let blocked = self
            .member_query_proof
            .calls
            .iter()
            .filter_map(|(sequence, (start, end, namespace))| {
                let invalidated = if *namespace {
                    self.member_query_proof.namespace_invalidated
                } else {
                    self.member_query_proof.receiver_invalidated
                };
                invalidated.then_some((*sequence, (*start, *end)))
            })
            .collect::<Vec<_>>();
        if blocked.is_empty() {
            return;
        }
        let sequences = blocked
            .iter()
            .map(|(sequence, _)| *sequence)
            .collect::<HashSet<_>>();
        let spans = blocked
            .iter()
            .map(|(_, span)| *span)
            .collect::<HashSet<_>>();
        let (calls, call_spans) = std::mem::take(&mut self.calls)
            .into_iter()
            .zip(std::mem::take(&mut self.call_spans))
            .filter(|(_, span)| !spans.contains(span))
            .unzip();
        self.calls = calls;
        self.call_spans = call_spans;
        self.confirmed_order
            .retain(|sequence| !sequences.contains(sequence));
        self.pending_calls
            .retain(|call| !sequences.contains(&call.seq));
        self.pending_spans
            .retain(|sequence, _| !sequences.contains(sequence));
    }
}
