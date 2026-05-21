use crate::tests::TestPlan;
use crate::tests::{GraphArgs, GraphFormat};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
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
}

pub(crate) fn run(args: GraphArgs) -> Result<ExitCode> {
    let content = fs::read_to_string(&args.plan)
        .with_context(|| format!("Failed to read plan from {}", args.plan.display()))?;
    let plan: TestPlan = serde_json::from_str(&content).context("Failed to parse plan JSON.")?;

    // 1. Collect all active nodes and edges from the plan reasons
    let mut changed_files = HashSet::new();
    let mut test_files = HashSet::new();
    let mut all_nodes = HashSet::new();
    let mut all_edges = HashSet::new(); // (from, to, via)

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
                    all_edges.insert((from, to, via));
                }
            }
        }
    }

    let mut sorted_nodes: Vec<String> = all_nodes.into_iter().collect();
    sorted_nodes.sort();

    let mut sorted_edges: Vec<(String, String, String)> = all_edges.into_iter().collect();
    sorted_edges.sort_by(|a, b| (&a.0, &a.1, &a.2).cmp(&(&b.0, &b.1, &b.2)));

    // 2. Render in the requested format
    let output = match args.format {
        GraphFormat::Json => {
            let mut nodes_json = Vec::new();
            for node in &sorted_nodes {
                let r#type = if changed_files.contains(node) {
                    "changed"
                } else if test_files.contains(node) {
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
            for (from, to, via) in &sorted_edges {
                edges_json.push(GraphEdgeJson {
                    from: from.clone(),
                    to: to.clone(),
                    via: via.clone(),
                });
            }

            serde_json::to_string_pretty(&GraphJson {
                nodes: nodes_json,
                edges: edges_json,
            })?
        }
        GraphFormat::Mermaid => {
            render_mermaid(&sorted_nodes, &sorted_edges, &changed_files, &test_files)
        }
    };

    if let Some(ref out_path) = args.out {
        fs::write(out_path, &output)
            .with_context(|| format!("Failed to write graph to {}", out_path.display()))?;
    } else {
        println!("{}", output);
    }

    Ok(ExitCode::SUCCESS)
}

fn escape_mermaid_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('|', "&#124;")
        .replace('\n', " ")
}

fn render_mermaid(
    nodes: &[String],
    edges: &[(String, String, String)],
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

        let label = if node.contains('"') {
            node.replace('"', "\\\"")
        } else {
            node.clone()
        };

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
        for (from, to, via) in edges {
            let from_id = node_ids.get(from).cloned().unwrap_or_else(|| from.clone());
            let to_id = node_ids.get(to).cloned().unwrap_or_else(|| to.clone());
            out.push_str(&format!(
                "    {} -->|{}| {}\n",
                from_id,
                escape_mermaid_label(via),
                to_id
            ));
        }
    }

    out
}
