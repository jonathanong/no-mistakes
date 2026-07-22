//! Renders a `flowchart LR` Mermaid diagram from the topology graph,
//! ported from `render-mermaid.mts`. Node ids are base64url-encoded so
//! arbitrary workflow/job/target strings become valid Mermaid identifiers;
//! labels are HTML-escaped for Mermaid's HTML-ish label syntax.

use super::model;

struct ConcurrentNode<'a> {
    source_id: String,
    concurrency: &'a model::WorkflowConcurrency,
}

pub fn render_workflow_topology_mermaid(topology: &model::WorkflowTopology) -> String {
    let mut lines = vec!["flowchart LR".to_string()];
    for workflow in &topology.workflows {
        lines.push(format!(
            "  subgraph {}[\"{}\"]",
            mermaid_id(&workflow.id),
            escape_label(&workflow.name)
        ));
        lines.push(format!(
            "    {}([\"{}\"])",
            workflow_node_id_from_path(&workflow.id),
            escape_label(&workflow.path)
        ));
        for job in jobs_for_workflow(topology, workflow) {
            let suffix = if matches!(job.kind, model::JobKind::MatrixTemplate) {
                " [matrix]"
            } else {
                ""
            };
            let label = job.name.as_deref().unwrap_or(job.key.as_str());
            lines.push(format!(
                "    {}[\"{}{suffix}\"]",
                job_node_id(&job.id),
                escape_label(label)
            ));
        }
        lines.push("  end".to_string());
    }

    for edge in &topology.edges {
        lines.extend(render_edge(edge));
    }

    let mut rendered_locks: std::collections::HashSet<String> = std::collections::HashSet::new();
    for declaration in concurrency_declarations(topology) {
        let lock_id = concurrency_lock_id(&declaration);
        if rendered_locks.insert(lock_id.clone()) {
            lines.push(format!(
                "  {lock_id}{{{{\"{}\"}}}}",
                escape_label(&format!(
                    "lock: {}",
                    declaration.concurrency.effective.group
                ))
            ));
        }
        lines.push(format!(
            "  {} -. \"{}\" .-> {lock_id}",
            declaration.source_id,
            escape_label(&concurrency_behavior(declaration.concurrency))
        ));
    }
    format!("{}\n", lines.join("\n"))
}

fn render_edge(edge: &model::WorkflowTopologyEdge) -> Vec<String> {
    match edge {
        model::WorkflowTopologyEdge::Needs(edge) => {
            vec![format!(
                "  {} --> {}",
                job_node_id(&edge.from),
                job_node_id(&edge.to)
            )]
        }
        model::WorkflowTopologyEdge::Artifact(edge) => vec![format!(
            "  {} -. \"{}\" .-> {}",
            job_node_id(&edge.from),
            escape_label(&format!(
                "artifact: {} [{}]",
                edge.name,
                edge.match_kind.as_str()
            )),
            job_node_id(&edge.to),
        )],
        model::WorkflowTopologyEdge::WorkflowRun(edge) => vec![format!(
            "  {} == workflow_run ==> {}",
            workflow_node_id_from_path(&edge.from),
            workflow_node_id_from_path(&edge.to),
        )],
        model::WorkflowTopologyEdge::Calls(edge) => {
            if let Some(to) = &edge.to {
                vec![format!(
                    "  {} -. calls .-> {}",
                    job_node_id(&edge.from),
                    workflow_node_id_from_path(to)
                )]
            } else {
                let remote_id = format!("remote_{}", hash_id(&edge.target));
                vec![
                    format!("  {remote_id}[\"{}\"]", escape_label(&edge.target)),
                    format!("  {} -. calls .-> {remote_id}", job_node_id(&edge.from)),
                ]
            }
        }
    }
}

fn jobs_for_workflow<'a>(
    topology: &'a model::WorkflowTopology,
    workflow: &model::WorkflowNode,
) -> Vec<&'a model::WorkflowJobNode> {
    topology
        .jobs
        .iter()
        .filter(|job| job.workflow_id == workflow.id)
        .collect()
}

fn concurrency_declarations(topology: &model::WorkflowTopology) -> Vec<ConcurrentNode<'_>> {
    let mut declarations = Vec::new();
    for workflow in &topology.workflows {
        if let Some(concurrency) = &workflow.concurrency {
            declarations.push(ConcurrentNode {
                source_id: workflow_node_id_from_path(&workflow.id),
                concurrency,
            });
        }
    }
    for job in &topology.jobs {
        if let Some(concurrency) = &job.concurrency {
            declarations.push(ConcurrentNode {
                source_id: job_node_id(&job.id),
                concurrency,
            });
        }
    }
    declarations
}

/// Literal (non-expression) groups sharing the same lowercased name share
/// one lock node; a group containing an unresolved `${{ }}` expression
/// gets its own lock per declaration, since two such groups can't be
/// proven to refer to the same runtime value.
fn concurrency_lock_id(declaration: &ConcurrentNode) -> String {
    let group = &declaration.concurrency.effective.group;
    let key = if group.contains("${{") {
        format!("declaration:{}", declaration.source_id)
    } else {
        format!("literal:{}", group.to_lowercase())
    };
    format!("lock_{}", hash_id(&key))
}

fn concurrency_behavior(concurrency: &model::WorkflowConcurrency) -> String {
    let cancel = match &concurrency.effective.cancel_in_progress {
        model::ConcurrencyValue::Bool(flag) => flag.to_string(),
        model::ConcurrencyValue::Text(text) => text.clone(),
    };
    format!("queue={}; cancel={cancel}", concurrency.effective.queue)
}

fn workflow_node_id_from_path(path: &str) -> String {
    format!("workflow_{}", hash_id(path))
}

fn job_node_id(id: &str) -> String {
    format!("job_{}", hash_id(id))
}

fn mermaid_id(id: &str) -> String {
    format!("workflow_group_{}", hash_id(id))
}

/// Base64url (unpadded, matching Node's `Buffer.toString("base64url")`) of
/// the UTF-8 bytes of `value` — used to turn arbitrary workflow/job/remote
/// target strings into valid Mermaid node identifiers. Hand-rolled rather
/// than pulling in a `base64` dependency for one call site.
fn hash_id(value: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
        let b2 = u32::from(*chunk.get(2).unwrap_or(&0));
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(n & 0x3F) as usize] as char);
        }
    }
    out
}

/// Order matters: `&` must be escaped first, or the entities this inserts
/// (`&amp;`, `&#35;`, ...) would themselves get re-escaped.
fn escape_label(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('#', "&#35;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
