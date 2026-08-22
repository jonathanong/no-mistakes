use super::diagnose_condition_references;
use crate::codebase::ci_graph::permissions::{PermissionSource, ResolvedPermissions};
use crate::codebase::workflow_topology::case_insensitive_lookup::CaseInsensitiveLookup;
use crate::codebase::workflow_topology::model::{
    DiagnosticCode, JobKind, StepKind, WorkflowJobNode, WorkflowStep,
};
use std::collections::{BTreeMap, HashSet};

fn permissions() -> ResolvedPermissions {
    ResolvedPermissions {
        source: PermissionSource::Default,
        scopes: BTreeMap::new(),
        assumed_default: true,
    }
}

fn job(key: &str, condition: Option<&str>, steps: Vec<WorkflowStep>) -> WorkflowJobNode {
    WorkflowJobNode {
        id: format!("ci.yml#{key}"),
        workflow_id: "ci.yml".into(),
        key: key.into(),
        kind: JobKind::Job,
        name: None,
        condition: condition.map(str::to_string),
        matrix: None,
        concurrency: None,
        steps,
        environment: None,
        timeout_minutes: None,
        runs_on: None,
        permissions: permissions(),
        outputs: None,
        env: None,
        secret_references: None,
    }
}

fn step(index: u32, id: Option<&str>, condition: Option<&str>) -> WorkflowStep {
    WorkflowStep {
        index,
        kind: StepKind::Run,
        id: id.map(str::to_string),
        name: None,
        condition: condition.map(str::to_string),
        uses: None,
        artifact: None,
        run: Some("echo".into()),
        with: None,
        env: None,
        secret_references: None,
    }
}

#[test]
fn condition_references_cover_needs_and_step_diagnostics() {
    let setup = job("setup", None, Vec::new());
    let setup_alias = job("Setup", None, Vec::new());
    let deploy = job(
        "deploy",
        Some("needs.missing && needs.setup"),
        vec![
            step(1, Some("prep"), None),
            step(2, Some("prep"), None),
            step(
                3,
                Some("later"),
                Some("steps.unknown && steps.prep && steps.later"),
            ),
        ],
    );
    let lookup = CaseInsensitiveLookup::new([
        (setup.key.clone(), &setup),
        (setup_alias.key.clone(), &setup_alias),
        (deploy.key.clone(), &deploy),
    ]);
    let mut diagnostics = Vec::new();
    diagnose_condition_references(
        &deploy,
        &HashSet::from(["ci.yml#setup".into()]),
        &lookup,
        &mut diagnostics,
    );
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();
    assert!(codes.contains(&DiagnosticCode::MissingNeedsDependency));
    assert!(codes.contains(&DiagnosticCode::DuplicateStepId));
    assert!(codes.contains(&DiagnosticCode::UnknownStepReference));
    assert!(codes.contains(&DiagnosticCode::NonPriorStepReference));
}
