use super::*;
use serde_json::json;
use std::sync::Arc;

pub(super) fn fixture(case: &str) -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &crate::test_support::rule_fixture_root("workflow-topology-policy")
            .join("secret-inventory")
            .join(case),
    )
}

pub(super) fn options(root: &Path, reports: Value) -> AnalyzeProjectOptions {
    serde_json::from_value(json!({
        "root": root, "config": ".no-mistakes.yml", "reports": reports,
    }))
    .unwrap()
}

#[test]
fn mixed_batched_topology_and_repeated_checks_share_one_owner_and_yaml_parse() {
    let fixture = fixture("repository");
    let root = fixture.path();
    let options = options(
        root,
        json!([
            {"type": "ciTopology"}, {"type": "check"}, {"type": "check"}, {"type": "ciTopology"},
        ]),
    );
    let context = AnalyzeProjectContext::prepare(&options).unwrap();
    let scope = context.scope(&options.reports[0], &options).unwrap();
    let owner = scope.ci_topology.as_ref().unwrap();
    let check = scope.check.as_ref().unwrap();
    assert!(Arc::ptr_eq(
        owner,
        check.prepared.workflow_documents.as_ref().unwrap()
    ));
    let sources = scope.traversal.source_store();
    assert!(Arc::ptr_eq(
        &sources,
        &check.prepared.visible_paths.source_store_for(root)
    ));
    assert_eq!(sources.yaml_parse_count(), 1);
    let canonical = owner.topology();
    let expected_topology = serde_json::to_value(canonical).unwrap();
    let reads = sources.physical_read_count();
    let parses = sources.yaml_parse_count();
    // Only the saved fixture's per-test copy disappears; reports must retain request-owned input.
    std::fs::remove_file(root.join(".github/workflows/check.yml")).unwrap();
    let mut reports = Vec::new();
    for request in &options.reports {
        let report = if request.report_type == "check" {
            context.project_report(request, &options)
        } else {
            context.command_report(request, &options)
        }
        .unwrap();
        reports.push(serde_json::from_str::<Value>(report.get()).unwrap());
    }
    assert_eq!(reports[0], expected_topology);
    assert_eq!(reports[3], expected_topology);
    assert_eq!(reports[1], reports[2]);
    assert_eq!(reports[1]["rules"].as_array().unwrap().len(), 3);
    assert!(std::ptr::eq(canonical, owner.topology()));
    assert_eq!(sources.physical_read_count(), reads);
    assert_eq!(sources.yaml_parse_count(), parses);
}

#[test]
fn topology_only_batches_prepare_documents_without_requiring_an_enabled_check() {
    let fixture = fixture("repository");
    let root = fixture.path();
    let mut options = options(
        root,
        json!([{"type": "ciTopology"}, {"type": "ciTopology"}]),
    );
    options.config = Some("disabled.yml".into());
    let context = AnalyzeProjectContext::prepare(&options).unwrap();
    let scope = context.scope(&options.reports[0], &options).unwrap();
    assert!(scope.check.is_none());
    assert!(scope
        .ci_topology
        .as_ref()
        .unwrap()
        .actions
        .documents
        .is_empty());
    let sources = scope.traversal.source_store();
    let reads = sources.physical_read_count();
    let first = context
        .command_report(&options.reports[0], &options)
        .unwrap();
    let second = context
        .command_report(&options.reports[1], &options)
        .unwrap();
    assert_eq!(first.get(), second.get());
    assert_eq!(sources.yaml_parse_count(), 1);
    assert_eq!(sources.physical_read_count(), reads);
}

#[test]
fn filtered_topology_projection_matches_loader_for_calls_cycles_and_malformed_documents() {
    for case in [
        "reusable",
        "unresolved",
        "noncallable",
        "cycle",
        "malformed",
    ] {
        let fixture = fixture(case);
        let root = fixture.path();
        let options = options(root, json!([{"type": "ciTopology"}]));
        let context = AnalyzeProjectContext::prepare(&options).unwrap();
        let scope = context.scope(&options.reports[0], &options).unwrap();
        let owner = scope.ci_topology.as_ref().unwrap();
        for filter in ["check.yml", "a.yml", "missing.yml", "../outside.yml"] {
            let filters = vec![filter.to_owned()];
            let expected = crate::codebase::workflow_topology::load_workflow_topology_from_parsed(
                root,
                &scope.traversal.config().ci,
                &owner.workflow_documents,
                &filters,
            );
            assert_eq!(
                owner.project_topology(&filters).as_ref(),
                &expected,
                "{case}: {filter}"
            );
            let request =
                serde_json::from_value(json!({"type": "ciTopology", "workflows": filters}))
                    .unwrap();
            let actual = context.command_report(&request, &options).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(actual.get()).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
        }
    }
}

#[test]
fn explicit_workflow_graph_reports_share_the_check_and_topology_owner() {
    let fixture = fixture("repository");
    let root = fixture.path();
    let options = options(
        root,
        json!([
            {"type": "check"},
            {"type": "ciTopology"},
            {"type": "dependencies", "files": [".github/workflows/check.yml"], "relationships": ["workflow"]},
        ]),
    );
    let context = AnalyzeProjectContext::prepare(&options).unwrap();
    let scope = context.scope(&options.reports[0], &options).unwrap();
    assert!(scope.traversal.build_plan().workflow_topology);
    let owner = scope.ci_topology.as_ref().unwrap();
    let prepared = scope
        .traversal
        .prepared_graph()
        .workflow_documents()
        .unwrap();
    assert!(
        matches!(prepared, crate::codebase::ci_workflows::PreparedWorkflowDocuments::Topology(value)
        if std::ptr::eq(value, owner.as_ref()))
    );
    let sources = scope.traversal.source_store();
    assert_eq!(sources.yaml_parse_count(), 1);
    let reads = sources.physical_read_count();
    context
        .graph_report(&options.reports[2], &options, Direction::Deps)
        .unwrap();
    context
        .project_report(&options.reports[0], &options)
        .unwrap();
    context
        .command_report(&options.reports[1], &options)
        .unwrap();
    assert_eq!(sources.yaml_parse_count(), 1);
    assert_eq!(sources.physical_read_count(), reads);
}
