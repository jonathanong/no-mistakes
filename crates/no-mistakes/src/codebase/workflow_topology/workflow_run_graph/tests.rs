use super::super::model::{
    DiagnosticCode, WorkflowNode, WorkflowRunEdge, WorkflowTopologyDiagnostic,
};
use super::diagnose_workflow_run_graph;

fn workflow(path: &str) -> WorkflowNode {
    WorkflowNode {
        id: path.into(),
        path: path.into(),
        name: path.into(),
        callable: false,
        workflow_call: None,
        triggers: Vec::new(),
        job_ids: Vec::new(),
        concurrency: None,
        env: None,
        secret_references: None,
    }
}

fn edge(from: &str, to: &str) -> WorkflowRunEdge {
    WorkflowRunEdge {
        from: from.into(),
        to: to.into(),
        types: None,
        branches: None,
        branches_ignore: None,
    }
}

fn diagnose(
    workflows: &[WorkflowNode],
    edges: &[WorkflowRunEdge],
) -> Vec<WorkflowTopologyDiagnostic> {
    let mut diagnostics = Vec::new();
    diagnose_workflow_run_graph(workflows, edges, &mut diagnostics);
    diagnostics
}

#[test]
fn two_node_cycle_ignores_edges_to_acyclic_neighbors() {
    let workflows = [workflow("a.yml"), workflow("b.yml"), workflow("c.yml")];
    let diagnostics = diagnose(
        &workflows,
        &[
            edge("a.yml", "b.yml"),
            edge("b.yml", "a.yml"),
            edge("a.yml", "c.yml"),
        ],
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunCycle)
    );
}

#[test]
fn acyclic_predecessor_of_a_cycle_does_not_join_the_scc() {
    let workflows = [workflow("a.yml"), workflow("b.yml"), workflow("c.yml")];
    let diagnostics = diagnose(
        &workflows,
        &[
            edge("a.yml", "b.yml"),
            edge("b.yml", "c.yml"),
            edge("c.yml", "b.yml"),
        ],
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunCycle)
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.code != DiagnosticCode::WorkflowRunChainLimit })
    );
}

#[test]
fn diamond_keeps_the_longest_chain_witness() {
    let workflows = [
        workflow("w0.yml"),
        workflow("w1.yml"),
        workflow("w2.yml"),
        workflow("w3.yml"),
        workflow("w4.yml"),
        workflow("w5.yml"),
    ];
    let diagnostics = diagnose(
        &workflows,
        &[
            edge("w0.yml", "w5.yml"),
            edge("w1.yml", "w2.yml"),
            edge("w2.yml", "w3.yml"),
            edge("w3.yml", "w4.yml"),
            edge("w4.yml", "w5.yml"),
        ],
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunChainLimit)
    );
}
