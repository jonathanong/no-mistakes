use super::super::PendingRelativeScope;
use super::kept_owner;

pub(super) fn count_unproven(pending: &PendingRelativeScope, kept: &[bool]) {
    let Some(observer) = crate::diagnostics::current().filter(|observer| observer.verbose()) else {
        return;
    };
    let count = pending
        .calls
        .iter()
        .filter(|call| pending.diagnostic_member_sequences.contains(&call.seq))
        .filter(|call| !kept_owner(kept, &call.owners))
        .count();
    if count > 0 {
        observer.increment("postgres.unproven_member_query_sql", count as u64);
    }
}
