use super::fixture;
use crate::codebase::rules::test_no_unmocked_dynamic_imports::resolve_mock_specifiers;
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use std::path::PathBuf;

#[test]
fn resolve_mock_specifiers_keeps_unresolved_specifier_keys() {
    let root = fixture();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let mocks = resolve_mock_specifiers(
        &["external".to_string()],
        &root.join("test.mts"),
        &resolver,
        None,
    );
    assert!(mocks.contains(&PathBuf::from("external")));
}
