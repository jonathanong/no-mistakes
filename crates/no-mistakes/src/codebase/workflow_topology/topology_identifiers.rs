//! Trivial id helpers, ported from `topology-identifiers.mts`.

/// A job or diagnostic id is `<workflow-path>#<job-key>`; a workflow's own
/// id is just its path (no `#`). Either way, the workflow path is
/// everything before the first `#`.
pub fn workflow_path_from_id(id: &str) -> &str {
    id.split_once('#').map_or(id, |(path, _)| path)
}
