use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::DepGraph;
use no_mistakes::codebase::test_discovery::TestExecutionTarget;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;

pub(crate) mod args;
pub(crate) mod audit;
pub(crate) mod changed_files;
pub(crate) mod comment;
pub(crate) mod config_invalidation;
pub(crate) mod configured_plan;
pub(crate) mod configured_plan_candidates;
pub(crate) mod diff_parser;
pub(crate) mod dotnet_dependency_changes;
pub(crate) mod git_diff;
pub(crate) mod graph;
pub(crate) mod impact;
mod impact_graph;
pub(crate) mod lockfile_changes;
pub(crate) mod package_manifest_changes;
pub(crate) mod plan;
pub(crate) mod plan_output;
pub(crate) mod prepared_plan;
pub(crate) mod swift_manifest_changes;
pub(crate) mod swift_resolved_changes;
pub(crate) mod targets;
pub(crate) mod why;

mod plan_finish;
mod plan_incompleteness;
pub use plan_incompleteness::{PlanIncompleteness, PlanIncompletenessKind};

pub use args::TestsArgs;
pub(crate) use args::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TestPlan {
    /// Complete normalized changed-file inventory used to prepare this plan.
    #[serde(default)]
    pub changed_files: Vec<String>,
    pub selected_tests: Vec<SelectedTest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<TestPlanGroupResult>,
    pub warnings: Vec<Warning>,
    pub fallback_triggered: bool,
    pub fallback_reason: Option<String>,
    /// Dependency coverage this plan could not finish. `fallback_triggered` is
    /// not a completeness guarantee: a scoped setup fallback can still be true
    /// while manifest or lockfile baselines are missing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incomplete: Vec<PlanIncompleteness>,
    /// Selected tests grouped by runner/config/project for CI command assembly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_targets: Vec<GroupedExecutionTarget>,
    /// Markdown comment, present when `includeComment` / `--include-comment` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct GroupedExecutionTarget {
    pub runner: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Path-prefix display name, such as a Swift package root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub base_command: Vec<String>,
    /// Runner flags without test file paths.
    pub runner_args: Vec<String>,
    pub test_files: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SelectedTest {
    pub test_file: String,
    pub confidence: Confidence,
    pub reasons: Vec<ImpactReason>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<TestExecutionTarget>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TestPlanGroupResult {
    pub r#type: String,
    pub selected: Vec<String>,
    pub remaining: usize,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low = 0,
    Medium = 1,
    High = 2,
}

impl Confidence {
    pub fn display_emoji(self) -> &'static str {
        match self {
            Confidence::Low => "🔴 Low",
            Confidence::Medium => "🟡 Medium",
            Confidence::High => "🟢 High",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ImpactReason {
    pub changed_file: String,
    pub path: Vec<String>,
    pub via: Vec<String>,
    /// Optional metadata for the corresponding `via` entry.  This stays
    /// absent for existing edge kinds so historical plan JSON is unchanged.
    #[serde(default, skip_serializing_if = "all_via_details_none")]
    pub via_details: Vec<Option<ImpactEdgeDetail>>,
}

fn all_via_details_none(details: &[Option<ImpactEdgeDetail>]) -> bool {
    details.iter().all(Option::is_none)
}

/// Debug provenance for an edge in a selected test-impact path.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ImpactEdgeDetail {
    Resource {
        consumer_file: String,
        call_sites: Vec<ResourceCallSite>,
    },
    VitestSetup {
        field: String,
    },
}

/// One static resource API call that contributed to a resource graph edge.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ResourceCallSite {
    pub call_kind: String,
    pub line: u32,
}

impl Ord for ResourceCallSite {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.line, &self.call_kind).cmp(&(other.line, &other.call_kind))
    }
}

impl PartialOrd for ResourceCallSite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub r#type: String,
    pub message: String,
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// The stable identity of a plan warning. Resource diagnostics are line-aware,
/// so two dynamic calls in the same consumer must remain distinct.
pub(crate) type WarningKey = (String, String, Option<u32>);

pub(crate) fn warning_key(warning: &Warning) -> WarningKey {
    (warning.r#type.clone(), warning.file.clone(), warning.line)
}

/// Add the stable warning projection for graph edges whose targets may be
/// unresolved at analysis time.
pub(crate) fn push_edge_warning(
    root: &Path,
    curr: &no_mistakes::codebase::dependencies::graph::NodeId,
    parent: &no_mistakes::codebase::dependencies::graph::NodeId,
    kind: no_mistakes::codebase::dependencies::graph::EdgeKind,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<WarningKey>,
) {
    use no_mistakes::codebase::dependencies::graph::EdgeKind;

    let (warning_type, message, file) = match kind {
        EdgeKind::DynamicImport | EdgeKind::ConditionalDynamicImport => {
            let file = plan::slash_node_name(curr, root);
            (
                "dynamic-import",
                format!("Dynamic import in `{}` might not be fully resolved.", file),
                file,
            )
        }
        EdgeKind::HttpCall => {
            let file = plan::slash_node_name(curr, root);
            (
                "http-call",
                format!(
                    "Dynamic HTTP call in `{}` to backend `{}`.",
                    file,
                    plan::slash_node_name(parent, root)
                ),
                file,
            )
        }
        EdgeKind::ProcessSpawn => {
            let file = plan::slash_node_name(curr, root);
            (
                "process-spawn",
                format!("Process spawned in `{}`.", file),
                file,
            )
        }
        _ => return,
    };
    let warning = Warning {
        r#type: warning_type.to_string(),
        message,
        file,
        line: None,
    };
    if warnings_seen.insert(warning_key(&warning)) {
        warnings.push(warning);
    }
}

/// Surface only resource diagnostics that belong to a changed file or an
/// already-selected graph path. Dynamic calls deliberately have no graph edge,
/// so they are warnings rather than a fallback signal.
pub(crate) fn push_resource_diagnostics(
    graph: &DepGraph,
    root: &Path,
    file: &Path,
    warnings: &mut Vec<Warning>,
    seen: &mut HashSet<WarningKey>,
) {
    let normalized = no_mistakes::codebase::ts_resolver::normalize_path(file);
    let relative = no_mistakes::codebase::ts_source::relative_slash_path(root, &normalized);
    for diagnostic in graph
        .resource_diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.consumer == normalized)
    {
        let line = u32::try_from(diagnostic.line).unwrap_or(u32::MAX);
        let (kind, what, fix) = match diagnostic.kind {
            no_mistakes::codebase::ts_resources::ResourceDiagnosticKind::DynamicPath => {
                ("dynamic-resource-path", "path", "use a literal path")
            }
            no_mistakes::codebase::ts_resources::ResourceDiagnosticKind::DynamicPattern => (
                "dynamic-resource-pattern",
                "pattern",
                "use a literal glob pattern",
            ),
            no_mistakes::codebase::ts_resources::ResourceDiagnosticKind::DynamicCwd => {
                ("dynamic-resource-cwd", "cwd", "use a literal cwd")
            }
        };
        let warning = Warning {
            r#type: kind.to_string(),
            message: format!(
                "Dynamic filesystem resource {what} in `{relative}` at line {line} cannot be resolved statically; {fix} or configure a test target trigger."
            ),
            file: relative.clone(),
            line: Some(line),
        };
        if seen.insert(warning_key(&warning)) {
            warnings.push(warning);
        }
    }
}

/// Keep optional edge provenance aligned with the public `via` path.  Omit
/// the field completely for ordinary paths so saved plan JSON remains stable.
pub(crate) fn via_details_from_edges(
    edges: &[no_mistakes::codebase::dependencies::graph::EdgeKind],
) -> Vec<Option<ImpactEdgeDetail>> {
    edges
        .iter()
        .map(|edge| match edge {
            no_mistakes::codebase::dependencies::graph::EdgeKind::VitestSetup(field) => {
                Some(ImpactEdgeDetail::VitestSetup {
                    field: field.as_str().to_string(),
                })
            }
            _ => None,
        })
        .collect()
}

pub fn run(args: TestsArgs) -> Result<ExitCode> {
    match args.command {
        TestsCommand::Plan(sub_args) => plan::run(*sub_args),
        TestsCommand::Targets(sub_args) => targets::run(sub_args),
        TestsCommand::Impact(sub_args) => impact::run(sub_args),
        TestsCommand::Why(sub_args) => why::run(sub_args),
        TestsCommand::Comment(sub_args) => comment::run(sub_args),
        TestsCommand::Graph(sub_args) => graph::run(sub_args),
        TestsCommand::Audit(sub_args) => audit::run(sub_args),
    }
}

const _: fn(TestsArgs) -> Result<ExitCode> = run;

pub(crate) use impact_graph::{
    build_test_impact_graph, build_test_impact_graph_for_reverse_query_with_prepared,
    ReverseQueryImpactRequest,
};

#[cfg(test)]
#[path = "mod_serde_tests.rs"]
mod serde_tests;
