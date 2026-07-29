use super::super::render_json::render_workflow_topology_json;
use super::super::{load_workflow_topology, load_workflow_topology_from_parsed};
use crate::codebase::ci_graph::impact::analyze_impact;
use crate::codebase::ci_graph::WorkflowSet;
use crate::codebase::ci_workflows::ParsedWorkflowSet;
use crate::config::v2::schema::CiConfig;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/workflow-topology/job-metadata"),
    )
}

#[test]
fn enriched_metadata_matches_the_byte_exact_schema_v1_golden() {
    let root = fixture();
    let topology = load_workflow_topology(&root, &CiConfig::default(), &[]);
    let actual = render_workflow_topology_json(&topology).unwrap();
    let expected = std::fs::read_to_string(root.join("expected.json")).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn enriched_metadata_reuses_prepared_workflow_documents() {
    let root = fixture();
    let config = CiConfig::default();
    let parsed = ParsedWorkflowSet::load(&root, &config);
    let prepared = load_workflow_topology_from_parsed(&root, &config, &parsed, &[]);
    let direct = load_workflow_topology(&root, &config, &[]);
    assert_eq!(prepared, direct);
}

#[test]
fn topology_permissions_exactly_match_ci_impact_permissions() {
    let root = fixture();
    let config = CiConfig::default();
    let parsed = ParsedWorkflowSet::load(&root, &config);
    let topology = load_workflow_topology_from_parsed(&root, &config, &parsed, &[]);
    let impact = analyze_impact(&WorkflowSet::from_parsed(&parsed), &["any.ts".to_string()]);

    let impact_permissions: BTreeMap<_, _> = impact
        .workflows
        .iter()
        .flat_map(|workflow| {
            workflow.jobs.iter().map(|job| {
                (
                    (workflow.path.clone(), job.id.clone()),
                    job.permissions.clone(),
                )
            })
        })
        .collect();

    for job in topology.jobs {
        assert_eq!(
            impact_permissions.get(&(job.workflow_id, job.key)),
            Some(&job.permissions)
        );
    }
}

#[test]
fn secret_references_are_static_names_attributed_to_the_direct_scope() {
    let topology = load_workflow_topology(&fixture(), &CiConfig::default(), &[]);
    let workflow = topology
        .workflows
        .iter()
        .find(|workflow| workflow.path.ends_with("default.yml"))
        .unwrap();
    assert_eq!(
        workflow.secret_references.as_deref(),
        Some(["Upper_Token".to_string(), "WORKFLOW_TOKEN".to_string()].as_slice())
    );

    let default_job = topology
        .jobs
        .iter()
        .find(|job| job.key == "default")
        .unwrap();
    assert_eq!(
        default_job.secret_references.as_deref(),
        Some(["BARE_JOB_TOKEN".to_string(), "Job_Token".to_string()].as_slice())
    );
    assert_eq!(
        default_job.steps[0].secret_references.as_deref(),
        Some(["STEP_ENV_TOKEN".to_string(), "STEP_RUN_TOKEN".to_string()].as_slice())
    );

    let dynamic_job = topology
        .jobs
        .iter()
        .find(|job| job.key == "override")
        .unwrap();
    assert_eq!(dynamic_job.steps[0].secret_references, None);
}

#[test]
fn absent_expression_text_has_no_static_context_references() {
    assert!(
        super::super::expression_references::static_context_references(None, "secrets", false)
            .is_empty()
    );
}
