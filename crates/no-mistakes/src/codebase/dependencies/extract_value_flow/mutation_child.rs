use super::*;

pub(super) fn record<'a>(
    members: &'a mut FxHashMap<String, ValueMutationHistory>,
    member: &str,
) -> &'a mut ValueMutationHistory {
    let occupied = members.contains_key(member);
    #[cfg(feature = "test-instrumentation")]
    metrics::child_lookup(occupied);
    if !occupied {
        #[cfg(feature = "test-instrumentation")]
        metrics::child_copy(member);
        members.insert(member.to_owned(), ValueMutationHistory::default());
    }
    members.get_mut(member).expect("receiver child exists")
}
