use super::super::PendingRelativeScope;
use super::kept_owner;

pub(super) fn unproven_spans(pending: &PendingRelativeScope, kept: &[bool]) -> Vec<(u32, u32)> {
    pending
        .calls
        .iter()
        .filter(|call| pending.diagnostic_member_sequences.contains(&call.seq))
        .filter(|call| !kept_owner(kept, &call.owners))
        .filter_map(|call| pending.call_spans.get(&call.seq).copied())
        .collect()
}
