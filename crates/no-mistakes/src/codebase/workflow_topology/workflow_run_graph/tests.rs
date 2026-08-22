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

#[test]
fn equal_length_over_limit_chains_keep_the_lexicographically_smaller_witness() {
    let workflows = [
        workflow("w0.yml"),
        workflow("w1.yml"),
        workflow("a.yml"),
        workflow("b.yml"),
        workflow("c.yml"),
        workflow("d.yml"),
        workflow("e.yml"),
        workflow("f.yml"),
        workflow("end.yml"),
    ];
    let diagnostics = diagnose(
        &workflows,
        &[
            edge("w0.yml", "a.yml"),
            edge("a.yml", "b.yml"),
            edge("b.yml", "c.yml"),
            edge("c.yml", "end.yml"),
            edge("w1.yml", "d.yml"),
            edge("d.yml", "e.yml"),
            edge("e.yml", "f.yml"),
            edge("f.yml", "end.yml"),
        ],
    );
    let chain = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunChainLimit)
        .expect("equal-length over-limit chains still diagnose");
    assert!(
        chain
            .message
            .contains("w0.yml -> a.yml -> b.yml -> c.yml -> end.yml")
    );
}

#[test]
fn three_node_cycle_with_a_chord_still_reports_a_witness() {
    let workflows = [workflow("a.yml"), workflow("b.yml"), workflow("c.yml")];
    let diagnostics = diagnose(
        &workflows,
        &[
            edge("a.yml", "b.yml"),
            edge("b.yml", "c.yml"),
            edge("c.yml", "a.yml"),
            edge("c.yml", "b.yml"),
        ],
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunCycle)
    );
}

#[test]
fn self_cycle_reports_a_two_node_witness() {
    let workflows = [workflow("loop.yml")];
    let diagnostics = diagnose(&workflows, &[edge("loop.yml", "loop.yml")]);
    let cycle = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunCycle)
        .expect("self-cycle");
    assert!(cycle.message.contains("loop.yml -> loop.yml"));
}

#[test]
fn search_skips_already_visited_scc_members_that_are_not_the_start() {
    let workflows = [
        workflow("a.yml"),
        workflow("b.yml"),
        workflow("c.yml"),
        workflow("d.yml"),
    ];
    let diagnostics = diagnose(
        &workflows,
        &[
            edge("a.yml", "b.yml"),
            edge("b.yml", "c.yml"),
            edge("c.yml", "a.yml"),
            edge("b.yml", "d.yml"),
            edge("d.yml", "b.yml"),
        ],
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunCycle)
    );
}
