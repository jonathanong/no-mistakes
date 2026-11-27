use super::config::Selection;
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::config::v2::load_v2_config;

#[test]
fn platform_consumption_follows_relays_but_not_execution_dependencies() {
    let root = super::tests::fixture("platform-reexports");
    let config = load_v2_config(&root, None).unwrap();
    let selection = Selection::new(
        &root,
        config
            .rule_applications(super::RULE_ID)
            .into_iter()
            .next()
            .unwrap(),
    )
    .unwrap();
    let entry = NodeId::symbol(root.join("src/named-entry.mts"), "Named");
    let origin = NodeId::symbol(root.join("src/named.mts"), "Named");
    let dependency = NodeId::symbol(root.join("src/control.mts"), "Unlisted");
    let graph = from_typed_maps(
        root,
        [
            (
                entry.clone(),
                vec![
                    (origin.clone(), EdgeKind::SymbolReexport),
                    (dependency.clone(), EdgeKind::Import),
                ],
            ),
            (
                origin.clone(),
                vec![(entry.clone(), EdgeKind::SymbolReexport)],
            ),
        ]
        .into_iter()
        .collect(),
        Default::default(),
    );
    let consumed = selection.platform_consumed_exports(&graph);
    assert!(consumed.contains(&entry));
    assert!(consumed.contains(&origin));
    assert!(!consumed.contains(&dependency));
}

#[test]
fn platform_exports_with_empty_normalized_root_reject_parent_escapes() {
    let select_in = |root: &str, file: &str| {
        let options = serde_json::from_value(serde_json::json!({
            "productionRoots": ["worker/src"],
            "testGlobs": ["tests/**"],
            "platformConsumedExports": [{ "file": file, "exports": ["CachedOrigin"] }],
        }))
        .unwrap();
        Selection::from_options(std::path::Path::new(root), options)
    };
    let select = |file: &str| select_in(".", file);
    assert!(select_in("../repo", "worker/src/index.mts").is_ok());
    assert!(select("worker/src/index.mts").is_ok());
    let error = select("../outside.ts").err().unwrap();
    assert!(error.to_string().contains("must stay within the root"));
}
