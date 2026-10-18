//! Callback revisit order must be independent of arena slot reuse.
use super::{CallbackIdentity, Value};
use crate::codebase::postgres::query_annotation::evaluate::frame_arena::ScopeArena;

pub(super) fn sort(callbacks: &mut [(CallbackIdentity, u8, Value)], scopes: &ScopeArena) {
    callbacks.sort_unstable_by(|(left, _, _), (right, _, _)| {
        // Slot tags protect identity; birth preserves main's allocation order.
        (&left.0, left.1, scopes.birth(left.2), &left.3).cmp(&(
            &right.0,
            right.1,
            scopes.birth(right.2),
            &right.3,
        ))
    });
}

#[cfg(test)]
mod tests;
