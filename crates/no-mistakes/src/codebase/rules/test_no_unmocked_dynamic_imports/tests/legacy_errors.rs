use super::*;
use crate::codebase::ts_resolver::load_tsconfig;

#[test]
fn check_inner_propagates_reachable_dep_disk_error() {
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let test_file = root.join("tests").join("bad.test.mts");
    let unreadable = root.join("src").join("unreadable.mts");
    let mut forward = HashMap::new();
    forward.insert(test_file.clone(), vec![unreadable.clone()]);
    let graph = from_raw_maps(root.clone(), forward, Default::default());
    let files = vec![test_file, unreadable];
    let manual_mocks = HashSet::new();
    let error =
        crate::codebase::rules::test_no_unmocked_dynamic_imports::standalone::legacy::check_inner(
            &root,
            &config,
            &files,
            &tsconfig,
            &graph,
            &manual_mocks,
        )
        .unwrap_err();
    assert!(error.to_string().contains("failed to read dependency file"));
}
