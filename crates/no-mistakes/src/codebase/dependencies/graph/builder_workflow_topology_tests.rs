use super::*;
use crate::codebase::ci_workflows::{
    ParsedWorkflowSet, PreparedWorkflowDocuments, PreparedWorkflowTopologyInputs,
};
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::config::v2::schema::CiConfig;
use std::borrow::Cow;

#[test]
fn workflow_graph_reuses_only_the_identical_prepared_projection() {
    let fixture = crate::test_support::materialize_saved_fixture(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/graph/workflow-topology-prepared"),
    );
    let root = fixture.path();
    let snapshot = VisiblePathSnapshot::new(root);
    let sources = snapshot.source_store_for(root);
    let ci = CiConfig::default();
    let documents = Arc::new(ParsedWorkflowSet::load_from_snapshot_and_sources(
        root, &ci, &snapshot, &sources,
    ));
    let inputs = PreparedWorkflowTopologyInputs::new(
        root,
        &ci,
        documents.clone(),
        ParsedWorkflowSet {
            documents: Vec::new(),
        },
    );
    let canonical = inputs.topology();
    let topology = workflow_topology_for_graph(
        root,
        &ci,
        &documents,
        Some(PreparedWorkflowDocuments::Topology(&inputs)),
    );
    assert!(matches!(&topology, Cow::Borrowed(value) if std::ptr::eq(*value, canonical)));
    let empty = ParsedWorkflowSet {
        documents: Vec::new(),
    };
    let narrower = workflow_topology_for_graph(
        root,
        &ci,
        &empty,
        Some(PreparedWorkflowDocuments::Topology(&inputs)),
    );
    assert!(matches!(&narrower, Cow::Owned(_)));
    assert!(narrower.workflows.is_empty());
    let legacy_documents: &ParsedWorkflowSet = &inputs;
    for legacy in [None, Some(PreparedWorkflowDocuments::Documents(&documents))] {
        let selected = legacy
            .map(PreparedWorkflowDocuments::documents)
            .unwrap_or(legacy_documents);
        let topology = workflow_topology_for_graph(root, &ci, selected, legacy);
        assert!(matches!(&topology, Cow::Owned(_)));
        assert_eq!(&*topology, canonical);
    }
}
