//! Renders the stable schema-v1 JSON, ported from `render-json.mts`.

use super::model::WorkflowTopology;

/// Compact serde JSON plus a trailing newline. Field names, field order, and
/// array/diagnostic sort order remain the schema-v1 stability contract
/// downstream consumers snapshot-diff — see the module docs on [`super`].
pub fn render_workflow_topology_json(topology: &WorkflowTopology) -> serde_json::Result<String> {
    Ok(format!("{}\n", serde_json::to_string(topology)?))
}
