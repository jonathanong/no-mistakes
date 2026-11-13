use super::*;
use std::sync::atomic::Ordering;

#[path = "stage_import_origins_test_support.rs"]
mod support;
use support::with_fixture;

#[test]
fn two_distinct_stage_exports_construct_one_reverse_origin_index() {
    with_fixture(&[], |flow, path, index| {
        assert_eq!(
            flow.cache
                .stage_import_origin_builds
                .load(Ordering::Relaxed),
            0
        );
        for name in ["first", "second", "first", "second"] {
            assert!(
                matches!(
                    flow.imported_stage_array(path, index, name),
                    Some(FlowValue::Array { .. })
                ),
                "{name}"
            );
        }
        assert_eq!(
            flow.cache
                .stage_import_origin_builds
                .load(Ordering::Relaxed),
            1
        );
        assert!(flow.cache.stage_import_origins.get().is_some());
    });
}

#[test]
fn ineligible_imports_do_not_construct_reverse_origin_index() {
    with_fixture(&[], |flow, path, index| {
        for name in ["missing", "mutable", "scalar"] {
            assert!(
                flow.imported_stage_array(path, index, name).is_none(),
                "{name}"
            );
        }
        assert_eq!(
            flow.cache
                .stage_import_origin_builds
                .load(Ordering::Relaxed),
            0
        );
        assert!(flow.cache.stage_import_origins.get().is_none());
    });
}

#[test]
fn named_alias_escape_blocks_only_its_export_origin() {
    with_fixture(&["escape", "barrel"], |flow, path, index| {
        assert!(flow.imported_stage_array(path, index, "first").is_none());
        assert!(flow.imported_stage_array(path, index, "second").is_some());
        assert_eq!(
            flow.cache
                .stage_import_origin_builds
                .load(Ordering::Relaxed),
            1
        );
    });
}

#[test]
fn namespace_cycles_expose_named_origins_without_widening_to_siblings() {
    with_fixture(&["namespace", "cycle-a", "cycle-b"], |flow, path, index| {
        assert!(flow.imported_stage_array(path, index, "first").is_none());
        assert!(flow.imported_stage_array(path, index, "second").is_some());
    });
}

#[test]
fn direct_namespace_exposes_every_local_including_unexported_bindings() {
    with_fixture(&["direct-namespace"], |flow, path, index| {
        assert!(flow.imported_stage_array(path, index, "first").is_none());
        let origins = flow.cache.stage_import_origins.get().unwrap();
        assert!(!origins.closed(&flow.inputs.root.join("stages.ts"), "hidden"));
    });
}

#[test]
fn missing_namespace_facts_and_unresolved_stars_block_every_origin() {
    for extra in [vec!["namespace"], vec!["unknown-namespace", "unknown-star"]] {
        with_fixture(&extra, |flow, path, index| {
            assert!(flow.imported_stage_array(path, index, "first").is_none());
            assert!(flow.imported_stage_array(path, index, "second").is_none());
            assert!(
                flow.cache
                    .stage_import_origins
                    .get()
                    .unwrap()
                    .unknown_namespace
            );
        });
    }
}

#[test]
fn named_namespace_imports_and_unresolved_imports_remain_ignored() {
    with_fixture(
        &["named-namespace", "namespace-barrel", "unresolved"],
        |flow, path, index| {
            assert!(flow.imported_stage_array(path, index, "first").is_some());
            assert!(flow.imported_stage_array(path, index, "second").is_some());
        },
    );
}

#[test]
fn unresolved_explicit_named_reexport_does_not_make_namespace_unknown() {
    with_fixture(
        &["unknown-named-namespace", "unknown-named"],
        |flow, path, index| {
            assert!(flow.imported_stage_array(path, index, "first").is_some());
            assert!(flow.imported_stage_array(path, index, "second").is_some());
        },
    );
}

#[test]
fn namespace_reexports_retain_target_file_wildcard() {
    with_fixture(
        &["nested-namespace", "namespace-barrel"],
        |flow, path, index| {
            assert!(flow.imported_stage_array(path, index, "first").is_none());
            assert!(flow.imported_stage_array(path, index, "second").is_none());
            let origins = flow.cache.stage_import_origins.get().unwrap();
            assert!(!origins.closed(&flow.inputs.root.join("stages.ts"), "hidden"));
        },
    );
}

#[test]
fn unsupported_bare_iterator_calls_remain_escaped() {
    with_fixture(&["bare-callee"], |flow, path, index| {
        assert!(flow.imported_stage_array(path, index, "first").is_none());
        assert!(flow.imported_stage_array(path, index, "second").is_some());
        assert_eq!(
            flow.cache
                .stage_import_origin_builds
                .load(Ordering::Relaxed),
            1
        );
    });
}
