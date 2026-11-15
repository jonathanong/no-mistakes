use super::{
    assemble, blank, branch_place, env_write, facts_for, hint, provenance, stored, Entry, Facts,
};
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::codebase::ts_source::module_facts::arguments::{PathProvenance, WriteScope};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

fn graph(edges: &[(&str, &[&str])]) -> DepGraph {
    let mut forward = FxHashMap::default();
    for (from, targets) in edges {
        let deps = targets
            .iter()
            .map(|target| (NodeId::file(*target), EdgeKind::Import))
            .collect();
        forward.insert(NodeId::file(*from), deps);
    }
    from_typed_maps(PathBuf::from("/repo"), forward, FxHashMap::default())
}

fn setup_write() -> PathProvenance {
    provenance(
        vec![env_write("ROOT", 5, 10, branch_place(true, None, None))],
        vec![],
    )
}

fn with_setup(local: PathProvenance) -> Facts {
    let mut facts = facts_for("/repo/test.mts", local);
    facts
        .0
        .insert("/repo/setup.mts".into(), stored(setup_write()));
    facts
}

fn message(edges: &[(&str, &[&str])], facts: &Facts, offset: Option<u32>) -> String {
    let linked = graph(edges);
    let owners = assemble(blank(&linked));
    let (_, cause, _) = owners.rank_environment_entry(
        Entry {
            file: "/repo/test.mts".into(),
            offset,
            line: 2,
            concrete: true,
        },
        hint("ROOT"),
        "has no proven assignment before reaching this scan",
        facts,
    );
    cause.message(Path::new("/repo"), "readdirSync")
}

#[test]
fn unknown_offset_abstains_when_the_test_file_may_also_write_the_variable() {
    let edges = &[("/repo/test.mts", &["/repo/setup.mts"][..])];
    let conditional = with_setup(provenance(
        vec![env_write("ROOT", 4, 20, branch_place(true, None, None))],
        vec![],
    ));
    let conditional_message = message(edges, &conditional, None);
    assert!(
        !conditional_message.contains("only inside a conditional"),
        "{conditional_message}"
    );
    assert!(
        !conditional_message.contains("setup.mts:5"),
        "{conditional_message}"
    );
    let unconditional = with_setup(provenance(
        vec![env_write("ROOT", 1, 4, WriteScope::Program)],
        vec![],
    ));
    let unconditional_message = message(edges, &unconditional, None);
    assert!(
        !unconditional_message.contains("setup.mts:5"),
        "{unconditional_message}"
    );
    let mut unnamed = provenance(vec![], vec![]);
    unnamed.env_unknown_write = true;
    let unnamed_message = message(edges, &with_setup(unnamed), None);
    assert!(
        !unnamed_message.contains("only inside a conditional"),
        "{unnamed_message}"
    );
    let mut missing = Facts(FxHashMap::default());
    missing
        .0
        .insert("/repo/setup.mts".into(), stored(setup_write()));
    let missing_message = message(edges, &missing, None);
    assert!(
        !missing_message.contains("setup.mts:5"),
        "{missing_message}"
    );
}

#[test]
fn unknown_offset_still_names_the_only_external_conditional() {
    let edges = &[("/repo/test.mts", &["/repo/setup.mts"][..])];
    let empty = with_setup(provenance(vec![], vec![]));
    let named = message(edges, &empty, None);
    assert!(named.contains("setup.mts:5"), "{named}");
    assert!(named.contains("only inside a conditional"), "{named}");
    let other = with_setup(provenance(
        vec![env_write("OTHER", 4, 20, branch_place(true, None, None))],
        vec![],
    ));
    let other_named = message(edges, &other, None);
    assert!(other_named.contains("setup.mts:5"), "{other_named}");
}

#[test]
fn non_script_imports_do_not_hide_the_one_script_assignment() {
    let edges = &[
        (
            "/repo/test.mts",
            &["/repo/bridge.mts", "/repo/catalog.json", "/repo/theme.css"][..],
        ),
        ("/repo/bridge.mts", &["/repo/setup.mts"][..]),
    ];
    let linked = graph(edges);
    let owners = assemble(blank(&linked));
    let test = Path::new("/repo/test.mts");
    let first = owners.related_files(test);
    assert_eq!(owners.related_files(test), first);
    assert_eq!(
        first,
        vec![
            PathBuf::from("/repo/bridge.mts"),
            PathBuf::from("/repo/setup.mts"),
        ]
    );
    let mut facts = with_setup(provenance(vec![], vec![]));
    facts.0.insert(
        "/repo/bridge.mts".into(),
        stored(provenance(vec![], vec![])),
    );
    let (_, cause, _) = owners.rank_environment_entry(
        Entry {
            file: "/repo/test.mts".into(),
            offset: Some(20),
            line: 3,
            concrete: true,
        },
        hint("ROOT"),
        "has no proven assignment before reaching this scan",
        &facts,
    );
    let named = cause.message(Path::new("/repo"), "readdirSync");
    assert!(named.contains("setup.mts:5"), "{named}");
    assert!(named.contains("only inside a conditional"), "{named}");
    let blocked = &[
        (
            "/repo/test.mts",
            &["/repo/bridge.mts", "/repo/writer.mts", "/repo/catalog.json"][..],
        ),
        ("/repo/bridge.mts", &["/repo/setup.mts"][..]),
    ];
    let blocked_message = message(blocked, &facts, Some(20));
    assert!(
        !blocked_message.contains("only inside a conditional"),
        "{blocked_message}"
    );
    assert!(
        !blocked_message.contains("setup.mts:5"),
        "{blocked_message}"
    );
}
