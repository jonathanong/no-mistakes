use super::super::ScopeVisitor;
use super::identity::Origin;
use super::{captures::QueryCapture, MemberCall};
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
use std::collections::HashSet;

fn uncertain(call: &MemberCall, proof: &super::MemberQueryProof) -> bool {
    call.origins
        .iter()
        .any(|origin| escape_blocks(origin, call, proof, blocks))
}

fn capture_uncertain(call: &MemberCall, proof: &super::MemberQueryProof) -> bool {
    call.origins
        .iter()
        .any(|origin| escape_blocks(origin, call, proof, capture_blocks))
}

fn escape_blocks(
    origin: &Origin,
    call: &MemberCall,
    proof: &super::MemberQueryProof,
    blocks_escape: fn(&super::Escape, &MemberCall, &Origin) -> bool,
) -> bool {
    let direct = proof.escapes.get(origin).is_some_and(|escapes| {
        escapes
            .iter()
            .any(|escape| blocks_escape(escape, call, origin))
    });
    direct
        || matches!(origin, Origin::Export(module, _)
            if escape_blocks(&Origin::Module(module.clone()), call, proof, blocks_escape))
}

fn blocks(escape: &super::Escape, call: &MemberCall, origin: &Origin) -> bool {
    // Another function, or a loop, can run before this member lookup.
    let ordered = escape.event < call.event || escape.owner != call.owner || escape.in_loop;
    ordered_escape(ordered, escape, origin)
}

fn capture_blocks(escape: &super::Escape, call: &MemberCall, _origin: &Origin) -> bool {
    // A top-level `const { query } = ns` copies the function during module
    // evaluation, before any function body runs. A later function does not
    // reach that copy. A copy inside a function is unordered with other
    // functions, and a loop still reaches backward. Function ids start at 1.
    let inside_function = call.owner != 0;
    let reaches = inside_function && (escape.owner != call.owner || escape.in_loop);
    escape.event < call.event || reaches
}

fn ordered_escape(ordered: bool, escape: &super::Escape, origin: &Origin) -> bool {
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
            if !self.member_query_proof.captures[index].trusted
                || capture_uncertain(&call, &self.member_query_proof)
            {
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
