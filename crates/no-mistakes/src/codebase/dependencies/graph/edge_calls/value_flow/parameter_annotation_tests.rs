use super::*;

#[path = "parameter_lifetime_tests.rs"]
pub(super) mod lifetime;

thread_local! {
    static ANNOTATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
pub(super) fn record_annotation() {
    lifetime::annotation();
    ANNOTATIONS.with(|count| count.set(count.get() + 1));
}
fn count() -> usize { ANNOTATIONS.with(std::cell::Cell::get) }

fn with_prepared_flow(
    visible_types: bool,
    run: impl FnOnce(&mut ValueFlowResolution<'_, '_>, &std::path::Path, &CallableFileIndex, &[ValueParameter]),
) {
    with_prepared_flow_in("test-cases/codebase-analysis/parameter-annotation-cache/fixture", visible_types, run);
}

fn with_prepared_flow_in(
    relative: &str,
    visible_types: bool,
    run: impl FnOnce(&mut ValueFlowResolution<'_, '_>, &std::path::Path, &CallableFileIndex, &[ValueParameter]),
) {
    use crate::codebase::ts_source::facts::{TsFactPlan, collect_ts_facts};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(relative);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let paths: Vec<_> = ["entry.ts", "barrel.ts", "types.ts"].into_iter().map(|file| root.join(file)).collect();
    let facts = collect_ts_facts(&paths, TsFactPlan { imports: true, function_calls: true, value_flow: true, ..Default::default() });
    let visible = if visible_types { paths.clone() } else { vec![paths[0].clone()] };
    let graph_files = GraphFiles::from_files(visible);
    let config = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&config);
    let inputs = GraphEdgeBuildInputs {
        root: &root, tsconfig: &config, tsconfig_catalog: None,
        plan: GraphBuildPlan { calls: true, ..Default::default() },
        workspace: None, graph_files: &graph_files, config_options: None,
        playwright_settings: &[], config_path: None, dotnet_facts: None,
        swift_facts: None, import_resolution_cache: None, visible_paths: None,
        workflow_documents: None, interner: std::sync::Arc::new(PathInterner::new()),
    };
    let indexes = CallableResolutionIndexes::default();
    let incoming = fx_map();
    let supers = fx_map();
    let cache = ParameterCache::default();
    let phases = execution_phases::ModuleWriterPhases::default();
    let mut flow = ValueFlowResolution {
        inputs: &inputs, facts: &facts, resolver: &resolver, indexes: &indexes,
        incoming: &incoming, supers: &supers, cache: &cache,
        replacements: None, replacement_read: false, foreign_acquisition: false,
        used_parameters: fx_set(), parameters_enabled: true, initializing: Vec::new(),
        visiting: fx_set(), calls_visiting: fx_set(), fields_visiting: fx_set(), default_origins: None,
        acquisition_context: None, invocation_read: None, invocation_start: None,
        executing_read: None, module_writer_phases: &phases, writer_dependencies: Default::default(),
        module_snapshot: None, member_initializer: None, member_read: None,
    };
    let path = &paths[0];
    let index = indexes.file(&facts, path).unwrap();
    let parameters = &facts.get_ts_facts(path).unwrap().value_flow.parameters;
    ANNOTATIONS.with(|count| count.set(0));
    run(&mut flow, path, &index, parameters);
}

fn has_query(value: &FlowValue) -> bool {
    match value {
        FlowValue::Provenance { executor, .. } => has_query(executor),
        FlowValue::TypedObject { properties, .. } => properties.contains_key("query"),
        _ => false,
    }
}

#[test]
fn parameter_annotation_cache_repeated_reads_compute_once() {
    with_prepared_flow(true, |flow, path, index, parameters| {
        let parameter = parameters.iter().find(|p| p.name == "db").unwrap();
        let first = flow.parameter(path, parameter, index);
        assert!(has_query(&first), "Unknown cannot satisfy the annotation work guard");
        for _ in 0..23 { assert!(flow.parameter(path, parameter, index) == first); }
        eprintln!("ACTUAL_PARAMETER_ANNOTATIONS reads=24 computations={}", count());
        assert_eq!(count(), 1, "eligible parameter annotation must be evaluated once per cache generation");
    });
}

#[test]
fn parameter_annotation_cache_keeps_parameters_negative_results_and_generations_separate() {
    with_prepared_flow(true, |flow, path, index, parameters| {
        let selected: Vec<_> = parameters.iter().filter(|p| matches!(p.name.as_str(), "db" | "other" | "missing" | "cycle")).collect();
        assert_eq!(selected.len(), 5);
        for parameter in &selected {
            let first = flow.parameter(path, parameter, index);
            assert_eq!(has_query(&first), matches!(parameter.name.as_str(), "db" | "other"));
            for _ in 0..3 { assert!(flow.parameter(path, parameter, index) == first); }
        }
        assert_eq!(count(), 5);
        let plain = parameters.iter().find(|p| p.name == "plain").unwrap();
        assert!(matches!(flow.parameter(path, plain, index), FlowValue::Unknown));
        assert_eq!(count(), 5);
        flow.cache.clear();
        flow.parameter(path, selected[0], index);
        assert_eq!(count(), 6, "a new incoming-call generation computes its own value");
    });
}

#[test]
fn parameter_annotation_cache_does_not_cross_resolver_catalog_boundaries() {
    let mut complete = None;
    with_prepared_flow(true, |flow, path, index, parameters| {
        let parameter = parameters.iter().find(|p| p.name == "db").unwrap();
        complete = Some(flow.parameter(path, parameter, index));
        assert!(has_query(complete.as_ref().unwrap()));
        assert_eq!(count(), 1);
    });
    with_prepared_flow(false, |flow, path, index, parameters| {
        let parameter = parameters.iter().find(|p| p.name == "db").unwrap();
        let first = flow.parameter(path, parameter, index);
        assert!(!has_query(&first));
        assert!(Some(first.clone()) != complete);
        for _ in 0..3 { assert!(flow.parameter(path, parameter, index) == first); }
        assert_eq!(count(), 1);
    });
}

#[test]
fn parameter_annotation_cache_does_not_annotate_disabled_or_recursive_reads() {
    with_prepared_flow(true, |flow, path, index, parameters| {
        let parameter = parameters.iter().find(|p| p.name == "db").unwrap();
        flow.parameters_enabled = false;
        assert!(matches!(flow.parameter(path, parameter, index), FlowValue::Unknown));
        assert_eq!(count(), 0);
        flow.parameters_enabled = true;
        let cache = flow.cache;
        let key = ParameterKey {
            file: path.to_path_buf(), scope: parameter.scope,
            name: parameter.name.clone(), invocation_read: flow.invocation_read.clone(),
            initializing: flow.initializing.clone(),
        };
        // A recursive acquisition must remain Unknown, even with an imported
        // annotation that would otherwise supply exact executor provenance.
        let CacheAccess::Compute(lease) = cache.acquire(key) else { panic!("cold parameter must acquire a lease"); };
        assert!(matches!(flow.parameter(path, parameter, index), FlowValue::Unknown));
        assert_eq!(count(), 0);
        drop(lease);
    });
}
