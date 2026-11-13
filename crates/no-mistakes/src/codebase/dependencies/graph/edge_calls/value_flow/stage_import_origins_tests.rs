use super::*;
use std::sync::atomic::Ordering;

fn with_fixture(
    extra: &[&str],
    run: impl FnOnce(&mut ValueFlowResolution<'_, '_>, &Path, &CallableFileIndex),
) {
    use crate::codebase::ts_source::facts::{TsFactPlan, collect_ts_facts};
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/stage-import-origin-cache/fixture"),
    );
    let paths: Vec<_> = ["entry", "stages"]
        .into_iter()
        .chain(extra.iter().copied())
        .map(|name| root.join(format!("{name}.ts")))
        .collect();
    let facts = collect_ts_facts(
        &paths,
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let files = GraphFiles::from_files(paths);
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&config);
    let inputs = GraphEdgeBuildInputs {
        root: &root,
        tsconfig: &config,
        tsconfig_catalog: None,
        plan: GraphBuildPlan {
            imports: true,
            calls: true,
            ..Default::default()
        },
        workspace: None,
        graph_files: &files,
        config_options: None,
        playwright_settings: &[],
        config_path: None,
        dotnet_facts: None,
        swift_facts: None,
        import_resolution_cache: None,
        visible_paths: None,
        workflow_documents: None,
        interner: std::sync::Arc::new(PathInterner::new()),
    };
    let indexes = CallableResolutionIndexes::default();
    let incoming = fx_map();
    let supers = fx_map();
    let cache = ParameterCache::default();
    let phases = execution_phases::ModuleWriterPhases::default();
    let mut flow = ValueFlowResolution {
        inputs: &inputs,
        facts: &facts,
        resolver: &resolver,
        indexes: &indexes,
        incoming: &incoming,
        supers: &supers,
        cache: &cache,
        replacements: None,
        replacement_read: false,
        foreign_acquisition: false,
        used_parameters: fx_set(),
        parameters_enabled: true,
        initializing: Vec::new(),
        visiting: fx_set(),
        calls_visiting: fx_set(),
        fields_visiting: fx_set(),
        default_origins: None,
        acquisition_context: None,
        invocation_read: None,
        invocation_start: None,
        executing_read: None,
        module_writer_phases: &phases,
        writer_dependencies: Default::default(),
        module_snapshot: None,
        member_initializer: None,
        member_read: None,
    };
    let path = root.join("entry.ts");
    let index = indexes.file(&facts, &path).unwrap();
    run(&mut flow, &path, &index);
}

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
