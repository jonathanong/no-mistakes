use std::path::PathBuf;

#[test]
fn check_json_snapshot_default_graph_empty_react_components() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-components/empty-proof/fixture");
    crate::check_json_snapshot::assert_saved_fixture(
        "default_graph_empty_react_components",
        &[(
            "default_relationships",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}
