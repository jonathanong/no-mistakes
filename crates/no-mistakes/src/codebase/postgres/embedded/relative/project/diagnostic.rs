use super::super::PendingRelativeScope;
use super::kept_owner;

pub(super) fn count_unproven(pending: &PendingRelativeScope, kept: &[bool]) -> u32 {
    let count = pending
        .calls
        .iter()
        .filter(|call| pending.diagnostic_member_sequences.contains(&call.seq))
        .filter(|call| !kept_owner(kept, &call.owners))
        .count() as u32;
    if count > 0 {
        if let Some(observer) = crate::diagnostics::current().filter(|observer| observer.verbose())
        {
            observer.increment("postgres.unproven_member_query_sql", count as u64);
        }
    }
    count
}
