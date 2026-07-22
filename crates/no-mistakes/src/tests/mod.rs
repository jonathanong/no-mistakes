use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use no_mistakes::codebase::test_discovery::TestExecutionTarget;
use no_mistakes::codebase::ts_resolver::TsConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;

pub(crate) mod args;
pub(crate) mod changed_files;
pub(crate) mod comment;
pub(crate) mod config_invalidation;
pub(crate) mod configured_plan;
pub(crate) mod configured_plan_candidates;
pub(crate) mod diff_parser;
pub(crate) mod git_diff;
pub(crate) mod graph;
pub(crate) mod impact;
pub(crate) mod lockfile_changes;
pub(crate) mod plan;
pub(crate) mod plan_output;
pub(crate) mod prepared_plan;
pub(crate) mod targets;
pub(crate) mod why;

pub use args::TestsArgs;
pub(crate) use args::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TestPlan {
    pub selected_tests: Vec<SelectedTest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<TestPlanGroupResult>,
    pub warnings: Vec<Warning>,
    pub fallback_triggered: bool,
    pub fallback_reason: Option<String>,
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
    }
}

const _: fn(TestsArgs) -> Result<ExitCode> = run;

/// Build the canonical graph shape shared by both test-impact entry points.
pub(crate) fn build_test_impact_graph(
    root: &Path,
    tsconfig: &TsConfig,
    include_symbols: bool,
) -> Result<DepGraph> {
    DepGraph::build_with_plan(
        root,
        tsconfig,
        GraphBuildPlan::test_impact().with_symbols(include_symbols),
    )
}

#[cfg(test)]
mod serde_tests {
    use super::*;

    #[test]
    fn impact_reason_resource_details_round_trip_and_legacy_details_are_optional() {
        let reason = ImpactReason {
            changed_file: "resources/schema.sql".to_string(),
            path: vec![
                "resources/schema.sql".to_string(),
                "src/load.ts".to_string(),
                "src/load.test.ts".to_string(),
            ],
            via: vec!["resource".to_string(), "dependency".to_string()],
            via_details: vec![
                Some(ImpactEdgeDetail::Resource {
                    consumer_file: "src/load.ts".to_string(),
                    call_sites: vec![ResourceCallSite {
                        call_kind: "read-file".to_string(),
                        line: 8,
                    }],
                }),
                None,
            ],
        };
        let json = serde_json::to_string(&reason).unwrap();
        assert!(json.contains("via_details"));
        assert_eq!(serde_json::from_str::<ImpactReason>(&json).unwrap(), reason);

        let legacy = r#"{"changed_file":"a.ts","path":["a.ts","a.test.ts"],"via":["dependency"]}"#;
        let legacy_reason: ImpactReason = serde_json::from_str(legacy).unwrap();
        assert!(legacy_reason.via_details.is_empty());
        assert!(!serde_json::to_string(&legacy_reason)
            .unwrap()
            .contains("via_details"));
    }
}
