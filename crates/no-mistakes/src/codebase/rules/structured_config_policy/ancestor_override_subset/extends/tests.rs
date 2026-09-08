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
    let mut parsed_ancestors = ParsedAncestorCache::default();
    let mut walk = Walk {
        root,
        nested_rel: ".oxlintrc.json",
        sources: &sources,
        assertion: &assertion,
        keys: &keys,
        stack: vec![root.to_path_buf(); MAX_EXTENDS_DEPTH],
        occurrences: 0,
        max_occurrences: MAX_EXTENDS_OCCURRENCES,
        parsed_ancestors: &mut parsed_ancestors,
        ancestors: Vec::new(),
        findings: &mut findings,
    };

    walk.follow(root, "./Cargo.toml");

    assert_eq!(walk.findings.len(), 1);
    assert!(walk.findings[0].message.contains("maximum depth"));
    assert!(walk.ancestors.is_empty());
}

#[test]
fn bounds_repeated_occurrences_in_a_fanout_graph() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy/ancestor-override-subset"),
    );
    let nested = root.join("diamond/nested/.oxlintrc.json");
    let files = [
        root.join("diamond/shared.json"),
        root.join("diamond/left.json"),
        root.join("diamond/right.json"),
        nested.clone(),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let source = crate::codebase::rules::read_source(&sources, &nested).unwrap();
    let value =
        crate::codebase::structured_value::parse_structured_value(&nested, &source).unwrap();
    let assertion = ValueAssertion::default();
    let keys = Keys::from_assertion(&assertion);
    let mut findings = Vec::new();
    let mut parsed_ancestors = ParsedAncestorCache::default();
    let mut walk = Walk {
        root: &root,
        nested_rel: "diamond/nested/.oxlintrc.json",
        sources: &sources,
        assertion: &assertion,
        keys: &keys,
        stack: vec![nested.clone()],
        occurrences: 0,
        max_occurrences: 3,
        parsed_ancestors: &mut parsed_ancestors,
        ancestors: Vec::new(),
        findings: &mut findings,
    };

    walk.visit(&nested, &value);

    assert_eq!(walk.ancestors.len(), 3);
    assert_eq!(walk.findings.len(), 1, "{:?}", walk.findings);
    assert!(walk.findings[0]
        .message
        .contains("maximum of 3 occurrences"));
}

#[test]
fn preserves_each_non_cycle_occurrence_in_a_diamond_extends_graph() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy/ancestor-override-subset"),
    );
    let nested = root.join("diamond/nested/.oxlintrc.json");
    let files = [
        root.join("diamond/shared.json"),
        root.join("diamond/left.json"),
        root.join("diamond/right.json"),
        nested.clone(),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let source = crate::codebase::rules::read_source(&sources, &nested).unwrap();
    let value =
        crate::codebase::structured_value::parse_structured_value(&nested, &source).unwrap();
    let assertion = ValueAssertion::default();
    let keys = Keys::from_assertion(&assertion);
    let mut findings = Vec::new();
    let mut parsed_ancestors = ParsedAncestorCache::default();
    let mut walk = Walk {
        root: &root,
        nested_rel: "diamond/nested/.oxlintrc.json",
        sources: &sources,
        assertion: &assertion,
        keys: &keys,
        stack: vec![nested.clone()],
        occurrences: 0,
        max_occurrences: MAX_EXTENDS_OCCURRENCES,
        parsed_ancestors: &mut parsed_ancestors,
        ancestors: Vec::new(),
        findings: &mut findings,
    };
    walk.visit(&nested, &value);
    assert!(walk.findings.is_empty(), "{:?}", walk.findings);
    assert_eq!(
        walk.ancestors
            .iter()
            .map(|ancestor| ancestor.rel.as_str())
            .collect::<Vec<_>>(),
        vec![
            "diamond/shared.json",
            "diamond/left.json",
            "diamond/shared.json",
            "diamond/right.json",
        ],
    );
    assert_eq!(
        walk.parsed_ancestors
            .parse_count(&root.join("diamond/shared.json")),
        1
    );
}
