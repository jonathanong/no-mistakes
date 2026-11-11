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
