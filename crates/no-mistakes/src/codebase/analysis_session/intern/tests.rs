use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::{intern_node_path, intern_node_str, NodeId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn hash_of(node: &NodeId) -> u64 {
    let mut hasher = DefaultHasher::new();
    node.hash(&mut hasher);
    hasher.finish()
}

fn path_os_eq(left: &Arc<Path>, right: &Arc<Path>) -> bool {
    left.as_os_str() == right.as_os_str()
}

#[test]
fn intern_hit_shares_arc_from_distinct_pathbufs() {
    let interner = PathInterner::new();
    let left = PathBuf::from("src/widget.ts");
    let right = PathBuf::from("src/widget.ts");
    assert!(!std::ptr::eq(left.as_os_str(), right.as_os_str()));

    let first = interner.intern_path(&left);
    let second = interner.intern_path(&right);
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.as_os_str(), Path::new("src/widget.ts").as_os_str());
}

#[test]
fn intern_path_miss_then_hit_covers_lookup_arms() {
    let interner = PathInterner::new();
    let miss = interner.intern_path("src/a.ts");
    let hit = interner.intern_path("src/a.ts");
    assert!(Arc::ptr_eq(&miss, &hit));
}

#[test]
fn intern_path_insert_occupied_and_vacant_are_distinct() {
    let interner = PathInterner::new();
    let vacant = interner.insert_path_arc(Arc::from(Path::new("src/vacant.ts")));
    let occupied = interner.insert_path_arc(Arc::from(Path::new("src/vacant.ts")));
    let other = interner.insert_path_arc(Arc::from(Path::new("src/other.ts")));
    assert!(Arc::ptr_eq(&vacant, &occupied));
    assert!(!Arc::ptr_eq(&vacant, &other));
    assert_eq!(vacant.as_os_str(), occupied.as_os_str());
}

#[test]
fn intern_str_miss_then_hit_and_insert_arms() {
    let interner = PathInterner::new();
    let owned: Arc<str> = Arc::from("send");
    let miss = interner.intern_str(owned.clone());
    let hit = interner.intern_str("send");
    assert!(Arc::ptr_eq(&miss, &hit));
    assert!(Arc::ptr_eq(&miss, &owned) || miss.as_ref() == owned.as_ref());

    let vacant = interner.insert_str_arc(Arc::from("vacant"));
    let occupied = interner.insert_str_arc(Arc::from("vacant"));
    let other = interner.insert_str_arc(Arc::from("other"));
    assert!(Arc::ptr_eq(&vacant, &occupied));
    assert!(!Arc::ptr_eq(&vacant, &other));
}

#[test]
fn intern_normalize_dot_dotdot_and_trailing_slash_share_arc() {
    let interner = PathInterner::new();
    let cases = [
        "src/a.ts",
        "src/./a.ts",
        "src/foo/../a.ts",
        "src/a.ts/",
        "src/./foo/../a.ts",
    ];
    let expected = interner.intern_path(cases[0]);
    for raw in cases {
        let interned = interner.intern_path(raw);
        assert!(
            Arc::ptr_eq(&expected, &interned),
            "normalize {raw} should share interned Arc"
        );
        assert_eq!(interned.as_os_str(), Path::new("src/a.ts").as_os_str());
    }
}

#[test]
fn types_node_id_file_vs_symbol_on_same_interned_path_stay_unequal() {
    let interner = PathInterner::new();
    let path = "src/a.ts";
    let file = NodeId::file_in(&interner, path);
    let symbol = NodeId::symbol_in(&interner, path, "job");
    let queue = NodeId::queue_job_in(&interner, path, "job");
    let module = NodeId::module_in(&interner, "job");
    let workflow = NodeId::workflow_job_in(&interner, path, "job");
    let step = NodeId::workflow_step_in(&interner, path, "job", 0);
    assert_ne!(file, symbol);
    assert_ne!(file, queue);
    assert_ne!(symbol, queue);
    assert_ne!(file, module);
    assert_ne!(queue, workflow);
    assert_ne!(workflow, step);
    assert_ne!(hash_of(&file), hash_of(&symbol));
    match (&file, &symbol) {
        (NodeId::File(left), NodeId::Symbol { file: right, .. }) => {
            assert!(Arc::ptr_eq(left, right));
        }
        other => panic!("expected File and Symbol, got {other:?}"),
    }
}

#[test]
fn session_a_and_session_b_do_not_share_interned_arcs() {
    let session_a = AnalysisSession::disabled();
    let session_b = AnalysisSession::disabled();
    let path_a = session_a.interner().intern_path("src/shared.ts");
    let path_b = session_b.interner().intern_path("src/shared.ts");
    let str_a = session_a.interner().intern_str("shared");
    let str_b = session_b.interner().intern_str("shared");
    assert!(!Arc::ptr_eq(&path_a, &path_b));
    assert!(!Arc::ptr_eq(&str_a, &str_b));
    assert!(path_os_eq(&path_a, &path_b));
    assert_eq!(str_a.as_ref(), str_b.as_ref());
    assert!(!std::ptr::eq(session_a.interner(), session_b.interner()));
}

#[test]
fn node_id_no_session_intern_byte_equals_session_interned() {
    let session = AnalysisSession::disabled();
    let interned_path = session.interner().intern_path("src/./widget.ts");
    let standalone_path = intern_node_path(PathBuf::from("src/widget.ts"));
    assert!(!Arc::ptr_eq(&interned_path, &standalone_path));
    assert_eq!(interned_path.as_os_str(), standalone_path.as_os_str());

    let interned_str = session.interner().intern_str("Widget");
    let standalone_str = intern_node_str("Widget");
    assert_eq!(interned_str.as_ref(), standalone_str.as_ref());

    let session_file = NodeId::file_in(session.interner(), "src/./widget.ts");
    let standalone_file = NodeId::file("src/widget.ts");
    assert_eq!(session_file, standalone_file);
    assert_eq!(hash_of(&session_file), hash_of(&standalone_file));

    let session_symbol = NodeId::symbol_in(session.interner(), "src/widget.ts", "Widget");
    let standalone_symbol = NodeId::symbol("src/./widget.ts", "Widget");
    assert_eq!(session_symbol, standalone_symbol);
    assert_eq!(hash_of(&session_symbol), hash_of(&standalone_symbol));
}

#[test]
fn intern_table_is_shared_across_session_handle_clones() {
    let session = AnalysisSession::disabled();
    let handle = session.interner_arc();
    let via_session = session.interner().intern_path("src/clone.ts");
    let via_handle = handle.intern_path("src/clone.ts");
    assert!(Arc::ptr_eq(&via_session, &via_handle));
}

#[test]
fn concurrent_intern_path_and_str_share_one_arc() {
    let interner = Arc::new(PathInterner::new());
    let path_workers: Vec<_> = (0..8)
        .map(|_| {
            let interner = Arc::clone(&interner);
            std::thread::spawn(move || interner.intern_path("src/parallel.ts"))
        })
        .collect();
    let str_workers: Vec<_> = (0..8)
        .map(|_| {
            let interner = Arc::clone(&interner);
            std::thread::spawn(move || interner.intern_str("parallel"))
        })
        .collect();
    let paths: Vec<_> = path_workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    let strings: Vec<_> = str_workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    for path in &paths {
        assert!(Arc::ptr_eq(&paths[0], path));
    }
    for value in &strings {
        assert!(Arc::ptr_eq(&strings[0], value));
    }
}
