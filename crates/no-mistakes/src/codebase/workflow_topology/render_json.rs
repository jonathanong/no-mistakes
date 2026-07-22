//! Renders the stable schema-v1 JSON, ported from `render-json.mts`.

use super::model::WorkflowTopology;

/// Pretty-printed (2-space indent, matching `JSON.stringify(x, undefined,
/// 2)`) with a trailing newline. This is the byte-for-byte stability
/// contract downstream consumers snapshot-diff — see the module docs on
/// [`super`].
pub fn render_workflow_topology_json(topology: &WorkflowTopology) -> serde_json::Result<String> {
    Ok(format!("{}\n", serde_json::to_string_pretty(topology)?))
}
