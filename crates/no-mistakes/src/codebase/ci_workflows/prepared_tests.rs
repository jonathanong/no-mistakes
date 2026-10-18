use super::*;
use crate::codebase::ts_source::VisiblePathSnapshot;

fn fixture() -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ci-prepared-secret-inventory"),
    )
}

fn config(root: &Path, name: &str) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(root, Some(&root.join(name))).unwrap()
}

#[test]
fn omitted_and_null_inventory_never_read_action_descriptors() {
    for name in ["disabled.yml", "null.yml"] {
        let fixture = fixture();
        let root = fixture.path();
        let snapshot = VisiblePathSnapshot::new(root);
        let sources = snapshot.source_store_for(root);
        let prepared = PreparedWorkflowTopologyInputs::for_config(
            root,
            &config(root, name),
            &snapshot,
            &sources,
        )
        .unwrap();
        assert!(prepared.actions.documents.is_empty());
        assert!(prepared.inventory_locations.get().is_none());
        assert_eq!(sources.physical_read_count(), 1);
        assert_eq!(sources.yaml_parse_count(), 1);
        assert!(
            prepared.topology.get().is_none(),
            "legacy consumers do not request topology"
        );
    }
}

#[test]
fn prepared_documents_topology_and_failures_survive_removing_the_sources() {
    let fixture = fixture();
    let root = fixture.path();
    let snapshot = VisiblePathSnapshot::new(root);
    let sources = snapshot.source_store_for(root);
    let configured = config(root, "inventory.yml");
    let prepared =
        PreparedWorkflowTopologyInputs::for_config(root, &configured, &snapshot, &sources).unwrap();
    assert_eq!(
        prepared
            .actions
            .documents
            .iter()
            .map(|document| document.path.as_str())
            .collect::<Vec<_>>(),
        ["actions/invalid/action.yaml", "actions/valid/action.yml"]
    );
    assert_eq!(prepared.actions.documents.len(), 2);
    assert_eq!(
        prepared
            .actions
            .documents
            .iter()
            .filter(|document| document.value.is_err())
            .count(),
        1
    );
    let reads = sources.physical_read_count();
    let parses = sources.yaml_parse_count();
    std::fs::remove_file(root.join(".github/workflows/ci.yml")).unwrap();
    std::fs::remove_file(root.join("actions/invalid/action.yaml")).unwrap();
    let first = prepared.topology();
    assert_eq!(first.jobs.len(), 1);
    assert!(std::ptr::eq(first, prepared.topology()));
    let again =
        PreparedWorkflowTopologyInputs::for_config(root, &configured, &snapshot, &sources).unwrap();
    for (again, prepared) in again
        .workflow_documents
        .documents
        .iter()
        .chain(&again.actions.documents)
        .zip(
            prepared
                .workflow_documents
                .documents
                .iter()
                .chain(&prepared.actions.documents),
        )
    {
        assert_eq!(again.path, prepared.path);
        assert_eq!(again.value, prepared.value);
    }
    assert_eq!(sources.physical_read_count(), reads);
    assert_eq!(sources.yaml_parse_count(), parses);
    assert!(prepared.matches_projection(root, &configured.ci, &prepared.workflow_documents));
    assert!(!prepared.matches_projection(
        root,
        &configured.ci,
        &ParsedWorkflowSet {
            documents: Vec::new()
        }
    ));
    let mut other_ci = configured.ci.clone();
    other_ci.workflow_dirs.push("other".into());
    assert!(!prepared.matches_projection(root, &other_ci, &prepared.workflow_documents));
    assert!(!prepared.matches_projection(
        &root.join("other"),
        &configured.ci,
        &prepared.workflow_documents
    ));
}

#[test]
fn absent_ci_demand_leaves_all_yaml_unread() {
    let fixture = fixture();
    let root = fixture.path();
    let snapshot = VisiblePathSnapshot::new(root);
    let sources = snapshot.source_store_for(root);
    assert!(PreparedWorkflowTopologyInputs::for_config(
        root,
        &NoMistakesConfig::default(),
        &snapshot,
        &sources,
    )
    .is_none());
    assert_eq!(sources.physical_read_count(), 0);
    assert_eq!(sources.yaml_parse_count(), 0);
}
