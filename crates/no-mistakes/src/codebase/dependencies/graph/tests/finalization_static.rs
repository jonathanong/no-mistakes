#[test]
fn edge_maps_and_edge_index_use_fx_hash() {
    let types = include_str!("../types.rs");
    assert!(
        types.contains("type EdgeMap = FxHashMap"),
        "graph EdgeMap must use rustc-hash FxHashMap"
    );
    let index = include_str!("../../../../edge_index.rs");
    assert!(
        index.contains("forward: FxHashMap") && index.contains("reverse: FxHashMap"),
        "EdgeIndex adjacency must use rustc-hash FxHashMap"
    );
}

#[test]
fn bfs_visited_sets_use_fx_hash() {
    let bfs = include_str!("../bfs.rs");
    assert!(
        bfs.contains("let mut visited: FxHashSet<&NodeId> = fx_set()"),
        "BFS visited set must use rustc-hash FxHashSet"
    );
    assert!(
        bfs.contains("let mut dynamic_import_files: FxHashSet<&NodeId> = fx_set()"),
        "BFS dynamic-import set must use rustc-hash FxHashSet"
    );
    assert!(
        bfs.contains("let mut result_idx: FxHashMap<&NodeId, usize> = fx_map()"),
        "BFS result index must use rustc-hash FxHashMap of borrowed NodeIds"
    );
    assert!(
        bfs.contains("let root_nodes: FxHashSet<NodeId>"),
        "BFS root-node set must use rustc-hash FxHashSet"
    );
    assert!(
        bfs.contains("file_universe: &crate::fx::PathSet")
            || bfs.contains("file_universe: &FxHashSet<PathBuf>"),
        "BFS file_universe must use rustc-hash PathSet/FxHashSet"
    );
}

#[test]
fn remaining_path_keyed_maps_use_fx_hash() {
    let graph_files = include_str!("../graph_files.rs");
    assert!(
        graph_files.contains("visible: Vec<u8>"),
        "GraphFiles.visible must store a dense FileId bitset instead of hashing PathBufs"
    );
    let types = include_str!("../types.rs");
    assert!(
        types.contains("type ResourceEdgeDetails = crate::fx::FxHashMap")
            || types.contains("type ResourceEdgeDetails = FxHashMap"),
        "ResourceEdgeDetails must use rustc-hash FxHashMap"
    );
    assert!(
        types.contains("universe: &crate::fx::PathSet")
            || types.contains("universe: &FxHashSet<PathBuf>"),
        "NodeId::is_in_file_universe must use rustc-hash PathSet/FxHashSet"
    );
    let facts = include_str!("../../../ts_source/facts.rs");
    assert!(
        facts.contains("facts: crate::codebase::ts_source::FileIdMap<TsFactSlot>"),
        "TsFactMap must intern facts in a FileIdMap"
    );
    let check_facts = include_str!("../../../check_facts/map.rs");
    assert!(
        check_facts.contains("ts: FileIdMap<Arc<CheckFileFacts>>"),
        "CheckFactMap.ts must intern facts in a FileIdMap"
    );
}

#[test]
fn leftover_path_keyed_maps_use_fx_hash() {
    let facts = include_str!("../../../ts_source/facts.rs");
    assert!(
        facts.contains("facts: crate::codebase::ts_source::FileIdMap<TsFactSlot>"),
        "TsFactMap must intern facts in a FileIdMap"
    );
    let types = include_str!("../types.rs");
    assert!(
        types.contains("type ResourceEdgeDetails = crate::fx::FxHashMap")
            || types.contains("type ResourceEdgeDetails = FxHashMap"),
        "ResourceEdgeDetails must use rustc-hash FxHashMap"
    );
    let cache = include_str!("../../../../ast/parsed_cache.rs");
    assert!(
        cache.contains("paths: FxHashMap<PathBuf, CachedModes>"),
        "ParsedProgramCache path index must use rustc-hash FxHashMap"
    );
}

#[test]
fn bfs_clones_neighbor_once_on_first_visit() {
    let bfs = include_str!("../bfs.rs");
    assert!(
        bfs.contains("node: neighbor.clone()"),
        "BFS must clone NodeId only when emitting an owned NodeEntry"
    );
    assert!(
        bfs.contains("visited.insert(neighbor)"),
        "BFS visited set must store borrowed NodeIds from the graph"
    );
    assert!(
        bfs.contains("queue.push_back((neighbor, next_depth))"),
        "BFS queue must store borrowed NodeIds instead of cloned owners"
    );
    assert!(
        !bfs.contains("let next = neighbor.clone();"),
        "BFS must not clone into a local just to clone that local again"
    );
    assert!(
        !bfs.contains("visited.insert(neighbor.clone())"),
        "BFS must not clone neighbor separately for visited insert"
    );
}

#[test]
fn lazy_import_clones_neighbor_once_on_first_visit() {
    let lazy = include_str!("../lazy_imports.rs");
    assert!(
        lazy.contains("neighbor.clone(),"),
        "lazy import traversal must clone NodeId once into the intern map"
    );
    assert!(
        lazy.contains("next_frontier.push(neighbor)"),
        "lazy import traversal must move the owned neighbor onto the next frontier"
    );
    assert!(
        !lazy.contains("visited.insert(next.clone())"),
        "lazy import traversal must not clone into visited, result, and result_idx separately"
    );
    assert!(
        !lazy.contains("result_idx.insert(next.clone()"),
        "lazy import traversal must not keep a cloned result_idx map"
    );
}
