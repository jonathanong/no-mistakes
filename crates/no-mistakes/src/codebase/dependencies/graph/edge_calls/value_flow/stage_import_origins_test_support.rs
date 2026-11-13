use super::*;

pub(super) fn with_fixture(
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
