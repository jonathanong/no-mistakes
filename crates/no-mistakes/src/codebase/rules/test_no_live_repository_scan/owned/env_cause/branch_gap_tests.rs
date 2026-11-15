use super::{
    assemble, blank, branch_place, covering_line, empty_graph, env_write, facts_for, file_hit,
    hint, provenance, Cover, Entry, FileHit,
};
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[test]
fn missing_facts_an_unnamed_mutation_or_a_describe_hook_stay_unnamed() {
    let mut poisoned = provenance(
        vec![env_write("ROOT", 6, 40, branch_place(true, None, None))],
        vec![],
    );
    poisoned.env_unknown_write = true;
    assert!(matches!(covering_line(&poisoned, "ROOT", 80), Cover::Many));
    assert!(matches!(file_hit(&poisoned, "ROOT"), FileHit::Blocked));
    poisoned.env_unknown_write = false;
    poisoned.env_opaque_write = true;
    assert!(matches!(covering_line(&poisoned, "ROOT", 80), Cover::Many));
    assert!(matches!(file_hit(&poisoned, "ROOT"), FileHit::Blocked));
    let describe = provenance(
        vec![env_write(
            "ROOT",
            8,
            30,
            branch_place(true, Some((40, 200)), None),
        )],
        vec![],
    );
    assert!(matches!(file_hit(&describe, "ROOT"), FileHit::Blocked));
    let file_wide = provenance(
        vec![env_write(
            "ROOT",
            4,
            20,
            branch_place(true, Some((0, 400)), None),
        )],
        vec![],
    );
    assert!(matches!(file_hit(&file_wide, "ROOT"), FileHit::Branch(4)));
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let facts = facts_for("/repo/test.mts", poisoned);
    let (_, cause, _) = owners.rank_environment_entry(
        Entry {
            file: "/repo/test.mts".into(),
            offset: Some(80),
            line: 9,
            concrete: true,
        },
        hint("ROOT"),
        "has no proven assignment before reaching this scan",
        &facts,
    );
    assert_eq!(cause.branch, None);
    let message = cause.message(Path::new("/repo"), "readdirSync");
    assert!(message.contains("test.mts:9 has no proven assignment"));
    assert!(!message.contains("only inside a conditional"));
    let setup = provenance(
        vec![env_write("ROOT", 5, 10, branch_place(true, None, None))],
        vec![],
    );
    let imported = facts_for("/repo/setup.mts", setup);
    let test = PathBuf::from("/repo/test.mts");
    let setup_path = PathBuf::from("/repo/setup.mts");
    let forward = FxHashMap::from_iter([(
        NodeId::file(&test),
        vec![(NodeId::file(&setup_path), EdgeKind::Import)],
    )]);
    let linked = from_typed_maps(PathBuf::from("/repo"), forward, FxHashMap::default());
    let linked_owners = assemble(blank(&linked));
    let (_, imported_cause, _) = linked_owners.rank_environment_entry(
        Entry {
            file: test,
            offset: Some(20),
            line: 4,
            concrete: true,
        },
        hint("ROOT"),
        "has no proven assignment before reaching this scan",
        &imported,
    );
    assert_eq!(imported_cause.branch, None);
    let imported_message = imported_cause.message(Path::new("/repo"), "readdirSync");
    assert!(!imported_message.contains("only inside a conditional"));
    assert!(!imported_message.contains("setup.mts:5"));
}
