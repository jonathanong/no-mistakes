use crate::tests::{GraphArgs, GraphFormat};
use crate::tests::{ImpactEdgeDetail, ResourceCallSite, TestPlan};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::process::ExitCode;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct GraphJson {
    pub nodes: Vec<GraphNodeJson>,
    pub edges: Vec<GraphEdgeJson>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct GraphNodeJson {
    pub name: String,
    pub r#type: String, // "changed", "test", or "intermediate"
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct GraphEdgeJson {
    pub from: String,
    pub to: String,
    pub via: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImpactEdgeDetail>,
}

pub(crate) fn run(args: GraphArgs) -> Result<ExitCode> {
    let content = fs::read_to_string(&args.plan)
        .with_context(|| format!("Failed to read plan from {}", args.plan.display()))?;
    let plan: TestPlan = serde_json::from_str(&content).context("Failed to parse plan JSON.")?;

    let output = match args.format {
        GraphFormat::Json => serde_json::to_string_pretty(&graph_json(&plan)?)?,
        GraphFormat::Mermaid => graph_mermaid(&plan)?,
    };
    crate::invocation::commit_timeout()?;

    if let Some(ref out_path) = args.out {
        fs::write(out_path, &output)
            .with_context(|| format!("Failed to write graph to {}", out_path.display()))?;
    } else {
        println!("{}", output);
    }

    Ok(ExitCode::SUCCESS)
}

const _: fn(GraphArgs) -> Result<ExitCode> = run;

pub(crate) fn graph_json(plan: &TestPlan) -> Result<GraphJson> {
    let parts = graph_parts(plan);
    let mut nodes_json = Vec::new();
    for node in &parts.sorted_nodes {
        let r#type = if parts.changed_files.contains(node) {
            "changed"
        } else if parts.test_files.contains(node) {
            "test"
        } else {
            "intermediate"
        };
        nodes_json.push(GraphNodeJson {
            name: node.clone(),
            r#type: r#type.to_string(),
        });
    }

    let mut edges_json = Vec::new();
    for (from, to, via, detail) in &parts.sorted_edges {
        edges_json.push(GraphEdgeJson {
            from: from.clone(),
            to: to.clone(),
            via: via.clone(),
            detail: detail.clone(),
        });
    }

    Ok(GraphJson {
        nodes: nodes_json,
        edges: edges_json,
    })
}

pub(crate) fn graph_mermaid(plan: &TestPlan) -> Result<String> {
    let parts = graph_parts(plan);
    Ok(render_mermaid(
        &parts.sorted_nodes,
        &parts.sorted_edges,
        &parts.changed_files,
        &parts.test_files,
    ))
}

struct GraphParts {
    sorted_nodes: Vec<String>,
    sorted_edges: Vec<(String, String, String, Option<ImpactEdgeDetail>)>,
    changed_files: HashSet<String>,
    test_files: HashSet<String>,
}

fn graph_parts(plan: &TestPlan) -> GraphParts {
    // 1. Collect all active nodes and edges from the plan reasons
    let mut changed_files = HashSet::new();
    let mut test_files = HashSet::new();
    let mut all_nodes = HashSet::new();
    let mut all_edges = BTreeMap::new(); // (from, to, via) -> debug provenance

    for test in &plan.selected_tests {
        test_files.insert(test.test_file.clone());
        for reason in &test.reasons {
            changed_files.insert(reason.changed_file.clone());

            for i in 0..reason.path.len() {
                all_nodes.insert(reason.path[i].clone());

                if i < reason.path.len() - 1 {
                    let from = reason.path[i].clone();
                    let to = reason.path[i + 1].clone();
                    let via = if i < reason.via.len() {
                        reason.via[i].clone()
                    } else {
                        "Dependency".to_string()
                    };
                    let detail = reason.via_details.get(i).cloned().flatten();
                    all_edges
                        .entry((from, to, via))
                        .and_modify(|existing| merge_edge_detail(existing, &detail))
                        .or_insert(detail);
                }
            }
        }
    }

    let mut sorted_nodes: Vec<String> = all_nodes.into_iter().collect();
    sorted_nodes.sort();

    let mut sorted_edges: Vec<(String, String, String, Option<ImpactEdgeDetail>)> = all_edges
        .into_iter()
        .map(|((from, to, via), detail)| (from, to, via, detail))
        .collect();
    sorted_edges.sort_by(|a, b| (&a.0, &a.1, &a.2).cmp(&(&b.0, &b.1, &b.2)));
    GraphParts {
        sorted_nodes,
        sorted_edges,
        changed_files,
        test_files,
    }
}

fn merge_edge_detail(existing: &mut Option<ImpactEdgeDetail>, incoming: &Option<ImpactEdgeDetail>) {
    let (
        Some(ImpactEdgeDetail::Resource {
            consumer_file: existing_consumer,
            call_sites: existing_sites,
        }),
        Some(ImpactEdgeDetail::Resource {
            consumer_file: incoming_consumer,
            call_sites: incoming_sites,
        }),
    ) = (existing.as_mut(), incoming)
    else {
        if existing.is_none() {
            *existing = incoming.clone();
        }
        return;
    };

    if existing_consumer != incoming_consumer {
        return;
    }
    merge_call_sites(existing_sites, incoming_sites);
}

fn merge_call_sites(existing: &mut Vec<ResourceCallSite>, incoming: &[ResourceCallSite]) {
    existing.extend(incoming.iter().cloned());
    existing.sort();
    existing.dedup();
}

fn escape_mermaid_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('|', "&#124;")
        .replace('\n', " ")
}

fn render_mermaid(
    nodes: &[String],
    edges: &[(String, String, String, Option<ImpactEdgeDetail>)],
    changed: &HashSet<String>,
    tests: &HashSet<String>,
) -> String {
    let mut out = String::new();
    out.push_str("graph TD\n");
    out.push_str("    classDef changed fill:#f96,stroke:#333,stroke-width:2px;\n");
    out.push_str("    classDef test fill:#9f6,stroke:#333,stroke-width:2px;\n\n");

    // Map node names to unique Mermaid IDs (e.g. n0, n1, ...)
    let mut node_ids = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        let id = format!("n{}", i);
        node_ids.insert(node.clone(), id.clone());

        let label = escape_mermaid_label(node);

        let style_class = if changed.contains(node) {
            ":::changed"
        } else if tests.contains(node) {
            ":::test"
        } else {
            ""
        };

        out.push_str(&format!("    {}[\"{}\"]{}\n", id, label, style_class));
    }

    if !edges.is_empty() {
        out.push('\n');
        for (from, to, via, detail) in edges {
            let from_id = node_ids.get(from).cloned().unwrap_or_else(|| from.clone());
            let to_id = node_ids.get(to).cloned().unwrap_or_else(|| to.clone());
            out.push_str(&format!(
                "    {} -->|{}| {}\n",
                from_id,
                escape_mermaid_label(&edge_label(via, detail.as_ref())),
                to_id
            ));
        }
    }

    out
}

fn edge_label(via: &str, detail: Option<&ImpactEdgeDetail>) -> String {
    match detail {
        Some(ImpactEdgeDetail::VitestSetup { field }) => format!("{} ({})", via, field),
        Some(ImpactEdgeDetail::Resource { .. }) => via.to_string(),
        None => via.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{
        Confidence, ImpactReason, ResourceCallSite, SelectedTest, TestPlan, Warning,
    };

    fn resource_detail(line: u32) -> ImpactEdgeDetail {
        ImpactEdgeDetail::Resource {
            consumer_file: "src/load-schema.ts".to_string(),
            call_sites: vec![ResourceCallSite {
                call_kind: "read-file-sync".to_string(),
                line,
            }],
        }
    }

    #[test]
    fn json_graph_keeps_and_merges_resource_provenance() {
        let reason = |line| ImpactReason {
            changed_file: "db/schema.sql".to_string(),
            path: vec![
                "db/schema.sql".to_string(),
                "src/load-schema.ts".to_string(),
                "tests/schema.test.ts".to_string(),
            ],
            via: vec!["resource".to_string(), "dependency".to_string()],
            via_details: vec![Some(resource_detail(line)), None],
        };
        let plan = TestPlan {
            selected_tests: vec![SelectedTest {
                test_file: "tests/schema.test.ts".to_string(),
                confidence: Confidence::High,
                reasons: vec![reason(3), reason(7)],
                targets: Vec::new(),
            }],
            groups: Vec::new(),
            warnings: Vec::<Warning>::new(),
            fallback_triggered: false,
            fallback_reason: None,
        };

        let graph = graph_json(&plan).unwrap();
        let resource = graph
            .edges
            .iter()
            .find(|edge| edge.via == "resource")
            .unwrap();
        let Some(ImpactEdgeDetail::Resource { call_sites, .. }) = &resource.detail else {
            panic!("resource detail must be retained");
        };
        assert_eq!(
            call_sites.iter().map(|site| site.line).collect::<Vec<_>>(),
            [3, 7]
        );
        assert!(!serde_json::to_string(&graph)
            .unwrap()
            .contains("\"detail\":null"));
    }

    #[test]
    fn legacy_reason_omits_empty_or_none_details() {
        let reason = ImpactReason {
            changed_file: "src/a.ts".to_string(),
            path: vec!["src/a.ts".to_string(), "tests/a.test.ts".to_string()],
            via: vec!["dependency".to_string()],
            via_details: vec![None],
        };
        let value = serde_json::to_value(reason).unwrap();
        assert!(value.get("via_details").is_none());
    }

    #[test]
    fn resource_call_sites_merge_by_line_then_kind() {
        let mut sites = vec![
            ResourceCallSite {
                call_kind: "read-file-sync".to_string(),
                line: 9,
            },
            ResourceCallSite {
                call_kind: "glob".to_string(),
                line: 3,
            },
        ];
        merge_call_sites(
            &mut sites,
            &[
                ResourceCallSite {
                    call_kind: "read-file".to_string(),
                    line: 3,
                },
                ResourceCallSite {
                    call_kind: "glob".to_string(),
                    line: 3,
                },
            ],
        );
        assert_eq!(
            sites
                .iter()
                .map(|site| (site.line, site.call_kind.as_str()))
                .collect::<Vec<_>>(),
            [(3, "glob"), (3, "read-file"), (9, "read-file-sync")]
        );
    }
}
