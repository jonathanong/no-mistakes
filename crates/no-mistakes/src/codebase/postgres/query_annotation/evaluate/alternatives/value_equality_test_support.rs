//! Test-only access to the real join; production visibility stays unchanged.
use super::super::Scope;
pub(in crate::codebase::postgres::query_annotation::evaluate) fn join(
    joined: &mut [Scope],
    current: &[Scope],
    original: &[Scope],
) {
    super::bindings::join_with_sql(joined, current, original, false);
}
