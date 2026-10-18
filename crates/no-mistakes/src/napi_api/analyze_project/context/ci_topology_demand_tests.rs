use super::ci_topology_tests::{fixture, options};
use super::*;
use serde_json::json;

#[test]
fn ordinary_noncheck_batches_do_not_prepare_ci_documents() {
    let fixture = fixture("repository");
    let options = options(fixture.path(), json!([{"type": "ciEnv", "var": "TOKEN"}]));
    let context = AnalyzeProjectContext::prepare(&options).unwrap();
    let scope = context.scope(&options.reports[0], &options).unwrap();
    assert!(scope.check.is_none());
    assert!(scope.ci_topology.is_none());
    assert!(!scope.traversal.build_plan().ci);
    assert!(!scope.traversal.build_plan().workflow_topology);
    assert_eq!(scope.traversal.source_store().yaml_parse_count(), 0);
}

#[test]
fn ci_and_workflow_graph_only_batches_declare_document_demand() {
    for relationship in ["ci", "workflow"] {
        let fixture = fixture("repository");
        let options = options(
            fixture.path(),
            json!([{
                "type": "dependencies", "files": [".github/workflows/check.yml"],
                "relationships": [relationship],
            }]),
        );
        let context = AnalyzeProjectContext::prepare(&options).unwrap();
        let scope = context.scope(&options.reports[0], &options).unwrap();
        assert!(scope.check.is_none());
        let owner = scope.ci_topology.as_ref().unwrap();
        assert_eq!(owner.workflow_documents.documents.len(), 1);
        assert!(owner.actions.documents.is_empty());
        assert_eq!(scope.traversal.source_store().yaml_parse_count(), 1);
        let prepared = scope
            .traversal
            .prepared_graph()
            .workflow_documents()
            .unwrap();
        assert!(matches!(prepared,
            crate::codebase::ci_workflows::PreparedWorkflowDocuments::Topology(prepared)
                if std::ptr::eq(prepared, owner.as_ref())));
    }
}

#[test]
fn prepared_ci_topology_rejects_wrong_option_types_and_unknown_fields() {
    for (report, expected) in [
        (
            json!({"type": "ciTopology", "workflows": false}),
            "invalid type: boolean",
        ),
        (
            json!({"type": "ciTopology", "unknownCiOption": true}),
            "unknownCiOption",
        ),
    ] {
        let fixture = fixture("repository");
        let options = options(fixture.path(), json!([report]));
        let context = AnalyzeProjectContext::prepare(&options).unwrap();
        let error = context
            .command_report(&options.reports[0], &options)
            .unwrap_err()
            .to_string();
        assert!(error.contains("invalid options JSON:"), "{error}");
        assert!(error.contains(expected), "{error}");
        assert!(
            !error.contains("prepared CI topology owner is missing"),
            "{error}"
        );
    }
}
