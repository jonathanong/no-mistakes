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

#[test]
fn runner_group_metadata_handles_supported_and_malformed_shapes() {
    use super::super::model::{WorkflowRunsOn, WorkflowRunsOnGroup, WorkflowRunsOnLabels};
    use super::super::workflow_values::parse_runs_on;

    let root = fixture();
    let source = std::fs::read_to_string(root.join("runs-on-cases.yml")).unwrap();
    let cases: serde_yaml::Value = serde_yaml::from_str(&source).unwrap();

    assert_eq!(
        parse_runs_on(cases.get("label-array")),
        Some(WorkflowRunsOn::Labels(vec![
            "self-hosted".to_string(),
            "linux".to_string(),
        ]))
    );
    assert_eq!(
        parse_runs_on(cases.get("group-label-array")),
        Some(WorkflowRunsOn::Group(WorkflowRunsOnGroup {
            group: "restricted".to_string(),
            labels: Some(WorkflowRunsOnLabels::Labels(vec![
                "self-hosted".to_string(),
                "linux".to_string(),
            ])),
        }))
    );
    assert_eq!(parse_runs_on(cases.get("missing-group")), None);
    assert_eq!(
        parse_runs_on(cases.get("invalid-labels")),
        Some(WorkflowRunsOn::Group(WorkflowRunsOnGroup {
            group: "restricted".to_string(),
            labels: None,
        }))
    );
    assert_eq!(parse_runs_on(cases.get("invalid-type")), None);
}

#[test]
fn nested_merges_supply_job_and_step_metadata_without_consuming_quoted_keys() {
    let root = crate::test_support::rule_fixture_root("workflow-topology-policy")
        .join("valid-nested-merges");
    let topology = load_workflow_topology(&root, &CiConfig::default(), &[]);
    assert!(
        topology.diagnostics.is_empty(),
        "{:?}",
        topology.diagnostics
    );
    let job = |file: &str| {
        topology
            .jobs
            .iter()
            .find(|job| job.key == "verify" && job.workflow_id.ends_with(file))
            .map(|job| serde_json::to_value(job).unwrap())
            .unwrap()
    };
    let merged_job = job("job.yml");
    assert_eq!(merged_job["timeoutMinutes"], 8);
    assert_eq!(merged_job["runsOn"], "ubuntu-latest");
    assert_eq!(merged_job["permissions"]["source"], "job");
    assert_eq!(merged_job["permissions"]["scopes"]["contents"], "read");
    assert_eq!(merged_job["steps"][0]["uses"], "example/build@v1");

    assert_eq!(job("step.yml")["steps"][1]["uses"], "example/build@v1");
    let quoted = job("quoted.yml");
    assert_eq!(quoted["steps"][1]["uses"], "example/build@v1");
    assert_eq!(quoted["steps"][1]["with"]["<<"], 7);
}

#[test]
fn unlocated_merge_reports_malformed_while_quoted_literal_keeps_graph() {
    let quoted_root = crate::test_support::rule_fixture_root("workflow-topology-policy")
        .join("merge-location-unavailable-quoted");
    let quoted = load_workflow_topology(&quoted_root, &CiConfig::default(), &[]);
    assert!(quoted.diagnostics.is_empty(), "{:?}", quoted.diagnostics);
    assert_eq!(quoted.jobs.len(), 1);
    let quoted_job = serde_json::to_value(&quoted.jobs[0]).unwrap();
    assert_eq!(quoted_job["env"]["<<"], "keep");

    let plain_root = crate::test_support::rule_fixture_root("workflow-topology-policy")
        .join("merge-location-unavailable");
    let plain = load_workflow_topology(&plain_root, &CiConfig::default(), &[]);
    assert_eq!(plain.diagnostics.len(), 1);
    assert!(
        plain.diagnostics[0]
            .message
            .contains("could not match a YAML `<<` key to its source location"),
        "{}",
        plain.diagnostics[0].message
    );
}
