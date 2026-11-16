use super::{
    alias, callers, export, graph, locate, namespace_import, resolution, resolve, site, Facts,
};
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::NodeId;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn reach_memo_repeats_a_miss() {
    let graph = graph(&[]);
    let reach = alias::Reach::new(&graph, &[]);
    let node = NodeId::file("/repo/src/idle.mts");
    assert!(!reach.reaches(&node));
    assert!(!reach.reaches(&node));
}

#[test]
fn a_classified_path_without_facts_uses_the_matching_direct_import() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from = root.join("src/lib.rs");
    let cargo = normalize_path(&root.join("Cargo.toml"));
    let other = root.join("src/main.rs");
    let resolution = resolution();
    let visible = PathSet::default();

    let linked = graph(&[(&from, &cargo)]);
    let facts = Facts {
        files: HashMap::new(),
    };
    let linked_locate = locate(&linked, &facts, &resolution, &visible);
    assert_eq!(
        resolve::resolve(&linked_locate, &from, "../Cargo.toml"),
        Some(cargo.clone())
    );

    let unlinked = graph(&[(&from, &other)]);
    let unlinked_locate = locate(&unlinked, &facts, &resolution, &visible);
    assert_eq!(
        resolve::resolve(&unlinked_locate, &from, "../Cargo.toml"),
        None
    );
    assert_eq!(
        resolve::resolve(&unlinked_locate, &from, "./no-such-registry-target.mts"),
        None
    );
}

#[test]
fn a_namespace_const_key_selects_an_exported_function_or_its_alias() {
    let caller = PathBuf::from("/repo/src/test.mts");
    let scan = PathBuf::from("/repo/src/scan.mts");
    let graph = graph(&[(&caller, &scan)]);
    let resolution = resolution();
    let visible = PathSet::default();
    let site = site(&caller);
    let caller_facts = TsFileFacts {
        imported_bindings: vec![namespace_import("./scan.mts")],
        ..TsFileFacts::default()
    };

    let function = Facts {
        files: HashMap::from([
            (caller.clone(), caller_facts.clone()),
            (
                scan.clone(),
                TsFileFacts {
                    exported_bindings: vec![export("sync", "sync", None)],
                    callable_scope_ids: vec![(CallableId(7), "sync".to_string())],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let function_locate = locate(&graph, &function, &resolution, &visible);
    let node = NodeId::callable(&scan, "sync", CallableId(7));
    assert_eq!(
        alias::function_node(
            &function_locate,
            &site,
            "workers",
            "sync",
            &callers(&scan, 7, "sync")
        ),
        Some(node)
    );

    let aliased = Facts {
        files: HashMap::from([
            (caller, caller_facts),
            (
                scan.clone(),
                TsFileFacts {
                    exported_bindings: vec![export("sync", "sync", None)],
                    callable_aliases: vec![super::callable_alias("sync", "processSync")],
                    callable_scope_ids: vec![(CallableId(9), "processSync".to_string())],
                    ..TsFileFacts::default()
                },
            ),
        ]),
    };
    let alias_locate = locate(&graph, &aliased, &resolution, &visible);
    let node = NodeId::callable(&scan, "processSync", CallableId(9));
    assert_eq!(
        alias::function_node(
            &alias_locate,
            &site,
            "workers",
            "sync",
            &callers(&scan, 9, "processSync")
        ),
        Some(node)
    );
}
