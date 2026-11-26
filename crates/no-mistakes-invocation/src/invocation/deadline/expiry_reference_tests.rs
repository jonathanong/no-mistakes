use super::Deadline;

// Exact eager pre-fix reference; keep its two independent snapshot checks.
/// Two snapshots that are both already expired. A commit or restore that
/// publishes between the loads returns no deadline.
pub(in crate::invocation::deadline) fn consecutive_expired(
    mut load: impl FnMut() -> Option<Deadline>,
) -> Option<Deadline> {
    let deadline = load()?;
    if crate::invocation::clock::now() < deadline.expires_at {
        return None;
    }
    let deadline = load()?;
    if crate::invocation::clock::now() < deadline.expires_at {
        return None;
    }
    Some(deadline)
}
