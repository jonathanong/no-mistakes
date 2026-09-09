#[test]
fn call_site_membership_uses_the_sorted_file_index() {
    let roots = include_str!("../edge_calls/roots.rs");
    let finish = include_str!("../builder_core/finish.rs");
    let sites = include_str!("../edge_calls/site_index.rs");
    assert!(
        sites.contains("fn index_sorted_call_sites_by_file("),
        "sorted call sites must be grouped once by file"
    );
    assert!(
        finish.contains("index_sorted_call_sites_by_file(&resolved_call_sites)"),
        "graph finish must build the per-file call-site index after sorting"
    );
    assert!(
        roots.contains("self.call_sites_by_file.contains_key(file)"),
        "has_call_site_in_file must probe the per-file index"
    );
    assert!(
        !roots.contains("site.file == file"),
        "has_call_site_in_file must not scan the whole-repo site list"
    );
}

#[test]
fn callable_nodes_group_scope_ids_once_per_file() {
    let finish = include_str!("../builder_core/finish.rs");
    assert!(
        finish.contains("ids_by_scope.entry(scope.as_str()).or_default().push(*id)"),
        "callable_scope_ids must be grouped once per file"
    );
    assert!(
        !finish.contains("candidate == scope"),
        "finish.rs must not rescan callable_scope_ids per display scope"
    );
}

#[test]
fn callable_node_lookup_uses_the_per_file_scope_index() {
    let collection = include_str!("../edge_calls/collection.rs");
    let helpers = include_str!("../edge_calls/index_build.rs");
    assert!(
        helpers.contains("fn index_scope_ids_by_display("),
        "callable_scope_ids must be grouped once by display scope"
    );
    assert!(
        collection.contains("index.unique_scope_id(scope)"),
        "call edges must look up the pre-indexed unique scope id"
    );
    assert!(
        !collection.contains("callable_scope_ids"),
        "callable_node_for_call must not scan callable_scope_ids per call"
    );
}

#[test]
fn unique_scope_id_is_none_when_display_names_collide() {
    use crate::codebase::dependencies::extract::CallableId;

    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_scope_ids: vec![
            (CallableId(1), "run".to_string()),
            (CallableId(2), "run".to_string()),
            (CallableId(3), "other".to_string()),
        ],
        ..Default::default()
    };
    let index = CallableFileIndex::from_facts(&facts);
    assert_eq!(index.unique_scope_id("other"), Some(CallableId(3)));
    assert_eq!(index.unique_scope_id("run"), None);
    assert_eq!(index.unique_scope_id("missing"), None);
}

fn test_call_site(file: &str) -> ResolvedCallSite {
    ResolvedCallSite {
        file: std::path::PathBuf::from(file),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
        source_callee: "f".to_string(),
        target: ResolvedCallTarget::Unknown,
    }
}

#[test]
fn sorted_call_site_index_groups_adjacent_files() {
    let sites = vec![
        test_call_site("/a.ts"),
        test_call_site("/a.ts"),
        test_call_site("/b.ts"),
    ];
    let index = index_sorted_call_sites_by_file(&sites);
    assert_eq!(
        index.get(std::path::Path::new("/a.ts")).cloned(),
        Some(0..2)
    );
    assert_eq!(
        index.get(std::path::Path::new("/b.ts")).cloned(),
        Some(2..3)
    );
    assert!(index_sorted_call_sites_by_file(&[]).is_empty());
}

#[test]
fn call_sites_in_file_returns_empty_for_unknown_paths() {
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        std::path::PathBuf::from("/root"),
        Default::default(),
        Default::default(),
    );
    assert!(graph
        .call_sites_in_file(std::path::Path::new("/missing.ts"))
        .is_empty());
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn criterion_adapters_exercise_indexed_construction() {
    use crate::codebase::dependencies::extract::{CallableAlias, CallableId};

    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        callable_bindings: vec![(0, "run".to_string(), CallableId(1))],
        callable_aliases: vec![CallableAlias {
            scope: None,
            scope_id: None,
            local: "alias".to_string(),
            target: "run".to_string(),
            binding_scope: 0,
            declared_at: 0,
            invalidated_at: None,
        }],
        ..Default::default()
    };
    assert!(benchmark_construct_callable_file_index(&facts) >= 2);
    assert_eq!(benchmark_probe_call_site_files(8), 8);
}
