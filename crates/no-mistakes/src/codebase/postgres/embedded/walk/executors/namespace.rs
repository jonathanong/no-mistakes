use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use std::collections::HashMap;

mod aggregate;
mod arguments;
mod identity;
mod proof;
use identity::Origin;

#[derive(Clone)]
pub(in crate::codebase::postgres::embedded::walk) struct MemberCall {
    pub(in crate::codebase::postgres::embedded::walk) sequence: u32,
    pub(super) span: (u32, u32),
    pub(super) origins: Vec<Origin>,
    pub(super) owner: u32,
    pub(super) event: u32,
    pub(in crate::codebase::postgres::embedded::walk) sql_looking: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Escape {
    pub(super) owner: u32,
    pub(super) event: u32,
    pub(super) in_loop: bool,
    pub(super) mutates_member: bool,
}

#[derive(Default)]
pub(crate) struct MemberQueryProof {
    pub(in crate::codebase::postgres::embedded::walk) scope_ids: Vec<u32>,
    pub(in crate::codebase::postgres::embedded::walk) next_scope_id: u32,
    pub(in crate::codebase::postgres::embedded::walk) function_ids: Vec<u32>,
    pub(in crate::codebase::postgres::embedded::walk) next_function_id: u32,
    pub(super) next_event: u32,
    pub(super) aliases: HashMap<(u32, String), Vec<Origin>>,
    pub(super) escapes: HashMap<Origin, Vec<Escape>>,
    captures: Vec<proof::QueryCapture>,
    captured_calls: Vec<proof::CapturedCall>,
    pub(in crate::codebase::postgres::embedded::walk) calls: Vec<MemberCall>,
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
                    None => self.imported_namespace_names.contains_key(name),
                }
            }
            Expression::CallExpression(call) => self.configured_require(call),
            Expression::ConditionalExpression(value) => {
                self.namespace_expression(&value.consequent)
                    || self.namespace_expression(&value.alternate)
            }
            _ => false,
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn namespace_query_valid(
        &self,
        _receiver: &Expression<'_>,
    ) -> bool {
        // An escape downgrades a recorded call. Rejecting here would drop it.
        self.query_binding
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_member_call(
        &mut self,
        sequence: u32,
        call: &CallExpression<'_>,
        receiver: &Expression<'_>,
    ) {
        let origins = self.receiver_origins(receiver);
        let owner = self
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        let event = self.member_query_proof.next_event;
        self.member_query_proof.next_event += 1;
        let sql_looking = self
            .pending_calls
            .iter()
            .rev()
            .find(|pending| pending.seq == sequence)
            .is_some_and(|pending| {
                pending
                    .call
                    .sql_text
                    .as_deref()
                    .is_some_and(super::super::scope::has_diagnostic_leading_statement)
                    || pending.call.variants.iter().any(|variant| {
                        super::super::scope::has_diagnostic_leading_statement(&variant.sql_text)
                    })
            });
        self.member_query_proof.calls.push(MemberCall {
            sequence,
            span: (call.span.start, call.span.end),
            origins,
            owner,
            event,
            sql_looking,
        });
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_query_receiver(
        &mut self,
        receiver: &Expression<'_>,
    ) {
        // Holders, returns, and aliases can replace `.query` after the copy.
        self.revoke_receiver(receiver, true);
    }

    /// Passing a binding into a function declared outside this file.
    fn revoke_passed_argument(&mut self, receiver: &Expression<'_>) {
        self.revoke_receiver(receiver, false);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_mutated_query_member(
        &mut self,
        receiver: &Expression<'_>,
    ) {
        self.revoke_receiver(receiver, true);
    }

    fn revoke_receiver(&mut self, receiver: &Expression<'_>, mutates_member: bool) {
        match unwrap_ts_wrappers(receiver) {
            Expression::ConditionalExpression(value) => {
                self.revoke_receiver(&value.consequent, mutates_member);
                self.revoke_receiver(&value.alternate, mutates_member);
                return;
            }
            Expression::LogicalExpression(value) => {
                self.revoke_receiver(&value.left, mutates_member);
                self.revoke_receiver(&value.right, mutates_member);
                return;
            }
            Expression::SequenceExpression(value) => {
                if let Some(last) = value.expressions.last() {
                    self.revoke_receiver(last, mutates_member);
                }
                return;
            }
            _ => {}
        }
        let origins = self.receiver_origins(receiver);
        let owner = self
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        let event = self.member_query_proof.next_event;
        self.member_query_proof.next_event += 1;
        for origin in origins {
            self.member_query_proof
                .escapes
                .entry(origin)
                .or_default()
                .push(Escape {
                    owner,
                    event,
                    in_loop: self.loop_depth > 0,
                    mutates_member,
                });
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_namespace_arguments(
        &mut self,
        callee: &Expression<'_>,
        arguments: &[Argument<'_>],
    ) {
        for (index, argument) in arguments.iter().enumerate() {
            if let Some(expression) = argument.as_expression() {
                self.revoke_namespace_expression(callee, index, expression);
            }
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_namespace_expression(
        &mut self,
        callee: &Expression<'_>,
        index: usize,
        expression: &Expression<'_>,
    ) {
        if self.namespace_argument_mutates(callee, index) {
            self.revoke_query_receiver(expression);
        } else {
            self.revoke_passed_argument(expression);
        }
    }
}
