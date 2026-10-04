//! Match relation identities for conservative dependency invalidation.

// An unqualified reference has no schema identity here: invalidate possible dependents
// conservatively. Fully qualified identities must still agree on their schema.
pub(super) fn names_match(left: &[String], right: &[String]) -> bool {
    let count = left.len().min(right.len());
    // decoded_parts always returns at least one part, even for an empty spelling.
    left[left.len() - count..] == right[right.len() - count..]
}
