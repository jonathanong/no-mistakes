use super::*;
use crate::codebase::ts_source::FileInventory;
use std::sync::Arc;

#[test]
fn follow_rejects_an_extends_chain_at_the_depth_limit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let resolved = root.join("Cargo.toml");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(std::slice::from_ref(
        &resolved,
    ))));
    let assertion = ValueAssertion::default();
    let keys = Keys::from_assertion(&assertion);
    let mut findings = Vec::new();
    let mut walk = Walk {
        root,
        nested_rel: ".oxlintrc.json",
        sources: &sources,
        assertion: &assertion,
        keys: &keys,
        stack: vec![root.to_path_buf(); MAX_EXTENDS_DEPTH],
        seen: HashSet::new(),
        ancestors: Vec::new(),
        findings: &mut findings,
    };

    walk.follow(root, "./Cargo.toml");

    assert_eq!(walk.findings.len(), 1);
    assert!(walk.findings[0].message.contains("maximum depth"));
    assert!(walk.ancestors.is_empty());
}
