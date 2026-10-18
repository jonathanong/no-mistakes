use super::super::ScopeVisitor;
use super::identity::Origin;
use super::MemberCall;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
use std::collections::HashSet;

fn uncertain(call: &MemberCall, proof: &super::MemberQueryProof) -> bool {
    call.origins.iter().any(|origin| {
        let escaped = |origin| {
            proof.escapes.get(origin).is_some_and(|escapes| {
                escapes.iter().any(|escape| {
                    escape.event < call.event || escape.owner != call.owner || escape.in_loop
                })
            })
        };
        escaped(origin)
            || matches!(origin, Origin::Export(module, _) if escaped(&Origin::Module(module.clone())))
    })
}

fn downgrade(call: &mut EmbeddedSqlCall) {
    call.kind = EmbeddedSqlKind::Dynamic;
    call.sql_text = None;
    call.variants.clear();
    call.sql_source_positions.clear();
    call.recovered_placeholder_positions.clear();
}

impl ScopeVisitor<'_> {
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
