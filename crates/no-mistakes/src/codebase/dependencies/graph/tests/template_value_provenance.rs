use super::*;
use receiver_escapes::{assert_export, assert_unknown};

#[test]
fn template_provenance_parts_are_not_runtime_array_elements() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("template-value-provenance"));
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let built = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    for file in ["string-spread", "string-index"] {
        assert_export(&built, &root, file, "replica.get");
    }
    assert_unknown(&built, &root, "array-control", "replica.get");
    for file in ["string-spread", "string-index", "array-control"] {
        assert_export(&built, &root, file, "unrelated.get");
    }
}
