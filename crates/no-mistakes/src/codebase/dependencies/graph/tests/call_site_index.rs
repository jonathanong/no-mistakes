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
