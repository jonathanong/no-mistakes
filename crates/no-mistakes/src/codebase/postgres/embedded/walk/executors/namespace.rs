use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use std::collections::HashMap;

mod aggregate;
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
        receiver: &Expression<'_>,
    ) -> bool {
        self.query_binding && self.receiver_query_valid(receiver)
    }

    pub(in crate::codebase::postgres::embedded::walk) fn receiver_query_valid(
        &self,
        receiver: &Expression<'_>,
    ) -> bool {
        !self
            .receiver_origins(receiver)
            .iter()
            .any(|origin| self.member_query_proof.escapes.contains_key(origin))
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
        let sql_looking = crate::diagnostics::current().is_some_and(|observer| observer.verbose())
            && self
                .pending_calls
                .iter()
                .rev()
                .find(|pending| pending.seq == sequence)
                .is_some_and(|pending| {
                    pending
                        .call
                        .sql_text
                        .as_deref()
                        .is_some_and(super::super::scope::has_known_leading_statement)
                        || pending.call.variants.iter().any(|variant| {
                            super::super::scope::has_known_leading_statement(&variant.sql_text)
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
        match unwrap_ts_wrappers(receiver) {
            Expression::ConditionalExpression(value) => {
                self.revoke_query_receiver(&value.consequent);
                self.revoke_query_receiver(&value.alternate);
                return;
            }
            Expression::LogicalExpression(value) => {
                self.revoke_query_receiver(&value.left);
                self.revoke_query_receiver(&value.right);
                return;
            }
            Expression::SequenceExpression(value) => {
                if let Some(last) = value.expressions.last() {
                    self.revoke_query_receiver(last);
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
                });
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_namespace_arguments(
        &mut self,
        call: &CallExpression<'_>,
    ) {
        for argument in &call.arguments {
            if let Some(expression) = argument.as_expression() {
                self.revoke_query_receiver(expression);
            }
        }
    }
}
