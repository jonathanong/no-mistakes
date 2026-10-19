use super::super::ScopeVisitor;
use super::identity::Origin;
use super::MemberCall;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
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

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn capture_is_uncertain(
        &self,
        receiver: &oxc_ast::ast::Expression<'_>,
    ) -> bool {
        let origins = self.receiver_origins(receiver);
        if origins.is_empty() {
            return false;
        }
        let owner = self
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        let call = MemberCall {
            sequence: 0,
            span: (0, 0),
            origins,
            owner,
            event: self.member_query_proof.next_event,
            sql_looking: false,
        };
        uncertain(&call, &self.member_query_proof)
    }

    pub(in crate::codebase::postgres::embedded::walk) fn downgrade_latest_confirmed_call(
        &mut self,
    ) {
        if let Some(call) = self.calls.last_mut() {
            downgrade(call);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn downgrade_escaped_member_calls(&mut self) {
        let blocked = self
            .member_query_proof
            .calls
            .iter()
            .filter(|call| uncertain(call, &self.member_query_proof))
            .collect::<Vec<_>>();
        if blocked.is_empty() {
            return;
        }
        let spans = blocked.iter().map(|call| call.span).collect::<HashSet<_>>();
        let sequences = blocked
            .iter()
            .map(|call| call.sequence)
            .collect::<HashSet<_>>();
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
