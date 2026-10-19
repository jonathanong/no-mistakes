use super::super::ScopeVisitor;
use super::identity::Origin;
use super::MemberCall;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use std::collections::HashSet;

fn uncertain(call: &MemberCall, proof: &super::MemberQueryProof) -> bool {
    call.origins
        .iter()
        .any(|origin| escape_blocks(origin, call, proof))
}

fn escape_blocks(origin: &Origin, call: &MemberCall, proof: &super::MemberQueryProof) -> bool {
    let direct = proof
        .escapes
        .get(origin)
        .is_some_and(|escapes| escapes.iter().any(|escape| blocks(escape, call, origin)));
    direct
        || matches!(origin, Origin::Export(module, _)
            if escape_blocks(&Origin::Module(module.clone()), call, proof))
}

fn blocks(escape: &super::Escape, call: &MemberCall, origin: &Origin) -> bool {
    let ordered = escape.event < call.event || escape.owner != call.owner || escape.in_loop;
    if !ordered {
        return false;
    }
    match origin {
        Origin::Export(_, _) => escape.mutates_member,
        Origin::Binding(_, _) | Origin::Module(_) => true,
    }
}

fn downgrade(call: &mut EmbeddedSqlCall) {
    call.kind = EmbeddedSqlKind::Dynamic;
    call.sql_text = None;
    call.variants.clear();
    call.sql_source_positions.clear();
    call.recovered_placeholder_positions.clear();
}

pub(super) struct QueryCapture {
    key: (u32, String),
    origins: Vec<Origin>,
    owner: u32,
    event: u32,
}

pub(super) struct CapturedCall {
    key: (u32, String),
    span: (u32, u32),
}

fn captured_call(capture: &QueryCapture) -> MemberCall {
    MemberCall {
        sequence: 0,
        span: (0, 0),
        origins: capture.origins.clone(),
        owner: capture.owner,
        event: capture.event,
        sql_looking: false,
    }
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn note_imported_alias_call(
        &mut self,
        call: &CallExpression<'_>,
    ) {
        let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else {
            return;
        };
        let Some(key) = self.capture_key(name.name.as_str()) else {
            return;
        };
        self.member_query_proof.captured_calls.push(CapturedCall {
            key,
            span: (call.span.start, call.span.end),
        });
    }

    fn capture_key(&self, name: &str) -> Option<(u32, String)> {
        let (index, _) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))?;
        Some((self.member_query_proof.scope_ids[index], name.to_string()))
    }

    /// The binding event is the call event. Escapes are applied after the walk,
    /// so a later other-function or loop write still reaches this capture.
    pub(in crate::codebase::postgres::embedded::walk) fn note_query_capture(
        &mut self,
        name: &str,
        receiver: &Expression<'_>,
    ) {
        let Some(key) = self.capture_key(name) else {
            return;
        };
        let owner = self
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        let origins = self.receiver_origins(receiver);
        let event = self.member_query_proof.next_event;
        self.member_query_proof.captures.push(QueryCapture {
            key,
            origins,
            owner,
            event,
        });
    }

    pub(in crate::codebase::postgres::embedded::walk) fn downgrade_escaped_member_calls(&mut self) {
        let blocked = self
            .member_query_proof
            .calls
            .iter()
            .filter(|call| uncertain(call, &self.member_query_proof))
            .map(|call| (call.span, call.sequence))
            .collect::<Vec<_>>();
        let mut spans = blocked
            .iter()
            .map(|(span, _)| *span)
            .collect::<HashSet<_>>();
        let sequences = blocked
            .iter()
            .map(|(_, sequence)| *sequence)
            .collect::<HashSet<_>>();
        let captures = self.member_query_proof.captures.len();
        let mut uncertain_keys = HashSet::new();
        for index in 0..captures {
            let call = captured_call(&self.member_query_proof.captures[index]);
            if uncertain(&call, &self.member_query_proof) {
                uncertain_keys.insert(self.member_query_proof.captures[index].key.clone());
            }
        }
        for captured in &self.member_query_proof.captured_calls {
            if uncertain_keys.contains(&captured.key) {
                spans.insert(captured.span);
            }
        }
        if spans.is_empty() && sequences.is_empty() {
            return;
        }
        for (call, span) in self.calls.iter_mut().zip(&self.call_spans) {
            if spans.contains(span) {
                downgrade(call);
            }
        }
        for pending in &mut self.pending_calls {
            if sequences.contains(&pending.seq) {
                downgrade(&mut pending.call);
            }
        }
    }
}
