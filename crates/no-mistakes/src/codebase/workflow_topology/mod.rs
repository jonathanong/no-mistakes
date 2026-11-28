//! GitHub Actions workflow topology graph (`no-mistakes ci topology`).
//!
//! Parses `.github/workflows/*.{yml,yaml}` into a typed graph — workflows,
//! jobs, and edges for `needs` control flow, reusable-workflow calls,
//! `workflow_run` subscriptions, and same-run artifact dataflow — plus
//! structured diagnostics for malformed, dangling, cyclic, or
//! contract-violating workflow definitions.
//!
//! This originated as a Rust port of a standalone TypeScript engine's
//! schema-v1 model and now carries first-party no-mistakes metadata. The
//! serialized [`model::WorkflowTopology`] JSON shape
//! (field names, field ORDER, and array/diagnostic sort order) is a
//! stability contract downstream consumers snapshot-diff against — see the
//! field-order notes in `model.rs` before changing anything there.
//!
//! Workflow directories come from [`CiConfig`] (shared with
//! [`super::ci_graph`]), defaulting to `.github/workflows`; discovery reuses
//! [`super::ci_graph::discover_workflow_files_from_snapshot`] rather than
//! walking the filesystem again.
//!
//! The original TS engine also ships a `WorkflowTopologyIndex` — a frozen
//! query object with ~20 traversal methods (transitive callers/callees,
//! `workflow_run` closures, artifact producer/consumer lookups). That index
//! is intentionally **not** ported here: it doesn't cross an N-API boundary
//! cleanly (it returns closures over frozen `Map`s), and its only consumer
//! today rebuilds a thin JS-side equivalent from this module's serialized
//! JSON. [`topology_graph::select_workflow_paths`] inlines the one
//! traversal (`--workflow` filter transitive-callee closure) this module
//! itself needs from that index.

pub mod artifact_download_resolver;
pub mod artifact_pattern_match;
pub mod artifact_resolution_diagnostics;
pub mod artifact_resolution_helpers;
pub mod artifact_resolution_types;
pub mod artifact_resolver;
pub mod artifact_run_context;
pub mod artifact_types;
pub mod artifact_values;
mod builder;
pub use builder::load_workflow_topology_from_parsed;
pub mod call_contract_diagnostics;
pub mod case_insensitive_lookup;
pub mod expression_references;
pub mod graph_algorithms;
pub mod graph_diagnostics;
pub mod impact;
pub mod model;
pub mod parse;
pub use crate::codebase::posix_path;
pub mod render_json;
pub mod render_mermaid;
pub mod secret_references;
pub mod topology_graph;
pub mod topology_identifiers;
pub mod value_primitives;
pub mod workflow_run_graph;
pub mod workflow_values;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

use crate::codebase::ci_workflows::ParsedWorkflowSet;
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::config::v2::schema::CiConfig;
use std::path::Path;

/// Loads the full workflow topology graph for a repository, building its
/// own visibility snapshot. Mirrors [`super::ci_graph::WorkflowSet::load`]'s
/// ergonomics; a caller that already has a snapshot (e.g. because it also
/// needs one to load config) should use
/// [`load_workflow_topology_from_snapshot`] instead to avoid discovering
/// the file universe twice in one invocation.
pub fn load_workflow_topology(
    root: &Path,
    ci: &CiConfig,
    workflow_filters: &[String],
) -> model::WorkflowTopology {
    let snapshot = VisiblePathSnapshot::new(root);
    load_workflow_topology_from_snapshot(root, ci, &snapshot, workflow_filters)
}

/// Parse-once-per-invocation variant of [`load_workflow_topology`] that
/// reuses a request-scoped [`VisiblePathSnapshot`] instead of building its
/// own. Discovers every workflow file under `ci.workflow_dirs`, parses each
/// one, then runs the full cross-file diagnostic and edge-resolution
/// pipeline in the same order as the TS engine's `loadWorkflowTopology`
/// (parse all → the `workflow_run` graph → local-call diagnostics →
/// call-contract diagnostics → call cycles → artifact graph → job graph →
/// duplicate names → `--workflow` filter validation → select + filter +
/// sort).
#[doc(hidden)]
pub fn load_workflow_topology_from_snapshot(
    root: &Path,
    ci: &CiConfig,
    snapshot: &VisiblePathSnapshot,
    workflow_filters: &[String],
) -> model::WorkflowTopology {
    let parsed = ParsedWorkflowSet::load_from_snapshot(root, ci, snapshot);
    load_workflow_topology_from_parsed(root, ci, &parsed, workflow_filters)
}
