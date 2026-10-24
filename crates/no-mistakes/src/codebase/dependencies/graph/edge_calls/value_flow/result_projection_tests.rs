use super::*;
use std::cell::{Cell, RefCell};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Observation {
    // These are shallow result Vec len/capacity/entry-size counts, not total heap or RSS.
    buffers: Vec<(usize, usize, usize)>,
    dependencies: FxHashMap<FunctionKey, FxHashSet<usize>>,
    previous: FxHashMap<usize, FxHashSet<FunctionKey>>,
    visits: FxHashMap<usize, usize>,
    incoming: Vec<ReplacementIncomingProbe>,
    raw_edges: Vec<String>,
}
thread_local! {
    static ORIGINAL: Cell<bool> = const { Cell::new(false) };
    static OBSERVED: RefCell<Observation> = RefCell::default();
}
pub(super) fn original() -> bool {
    ORIGINAL.get()
}
pub(super) fn record_buffer<T>(results: &[T], capacity: usize) {
    OBSERVED.with_borrow_mut(|observed| {
        observed
            .buffers
            .push((results.len(), capacity, size_of::<T>()))
    });
}
pub(super) fn record_dependencies(
    dependencies: &FxHashMap<FunctionKey, FxHashSet<usize>>,
    previous: &FxHashMap<usize, FxHashSet<FunctionKey>>,
    visits: &FxHashMap<usize, usize>,
) {
    OBSERVED.with_borrow_mut(|observed| {
        observed.dependencies = dependencies.clone();
        observed.previous = previous.clone();
        observed.visits = visits.clone();
    });
}
fn with_original<T>(original: bool, run: impl FnOnce() -> T) -> (T, Observation) {
    struct Restore(bool, Observation);
    impl Drop for Restore {
        fn drop(&mut self) {
            ORIGINAL.set(self.0);
            OBSERVED.replace(std::mem::take(&mut self.1));
        }
    }
    let _restore = Restore(
        ORIGINAL.replace(original),
        OBSERVED.replace(Observation::default()),
    );
    (run(), OBSERVED.with_borrow(Clone::clone))
}

fn with_fixture(
    name: &str,
    run: impl FnOnce(&GraphEdgeBuildInputs<'_>, &dyn TsFactLookup, &dyn ImportResolution),
) {
    use crate::codebase::ts_source::facts::{TsFactPlan, collect_ts_facts};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join(name)
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let paths = crate::codebase::ts_source::discover_files(&root, &[]);
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
    run(&inputs, &facts, &resolver);
}

type Outcome = (Vec<ResolvedCallSite>, Vec<Edge>, [usize; 3]);
fn run(name: &str, original: bool) -> (Outcome, Observation) {
    with_original(original, || {
        let mut result = None;
        with_fixture(name, |inputs, facts, resolver| {
            let pass =
                collect_call_edges_for_core(inputs, facts, resolver, &mut fx_map(), &mut fx_map());
            let probe =
                crate::codebase::dependencies::graph::value_flow_metrics::replacement_snapshot();
            OBSERVED.with_borrow_mut(|observed| {
                observed.incoming = probe.incoming;
                observed.raw_edges = probe.admission_raw_edges;
            });
            let work = [
                pass.parameter_cache.computations(),
                pass.parameter_cache.execution_steps(),
                pass.parameter_cache.module_provider_checks(),
            ];
            result = Some((pass.sites, pass.edges, work));
        });
        result.unwrap()
    })
}

#[test]
fn compact_result_buffer_reduces_actual_entry_backing_without_skipping_work() {
    let original = std::env::var_os("NO_MISTAKES_RESULT_BUFFER_ORIGINAL").is_some();
    let ((sites, _, _), observed) = run("value-flow-replacement-scale-16", original);
    assert_eq!(sites.len(), 16);
    assert!(!observed.buffers.is_empty());
    for &(len, capacity, entry_size) in &observed.buffers {
        println!(
            "RESULT_BUFFER_WORK len={len} capacity={capacity} entry_size={entry_size} shallow_bytes={}",
            capacity * entry_size
        );
        assert!(len > 0);
        assert!(entry_size < size_of::<(usize, PreviousResolution, FxHashSet<FunctionKey>)>());
        assert!(
            capacity * entry_size
                < capacity * size_of::<(usize, PreviousResolution, FxHashSet<FunctionKey>)>(),
            "old result backing budget"
        );
    }
}

#[test]
fn compact_results_preserve_sites_raw_edges_dependencies_and_work() {
    for name in [
        "value-flow-replacement-cold",
        "inline-member-precision-no-writes",
        "value-flow-replacement-scale-16",
        "value-flow-replacement-scale-256",
        "value-flow-foreign-parameter-timing",
        "value-flow-static-call-admission",
        "value-flow-repeated-replacement-controls",
        "value-flow-replacement-uncertain-execution",
    ] {
        let (actual, mut new) = run(name, false);
        let (expected, mut old) = run(name, true);
        assert_eq!(
            actual, expected,
            "complete sites, raw edge sequence and work: {name}"
        );
        assert_eq!(
            new.buffers.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>(),
            old.buffers.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>(),
            "same batch cardinality: {name}"
        );
        new.buffers.clear();
        old.buffers.clear();
        assert_eq!(
            new, old,
            "dependency owners, prior links and every visit: {name}"
        );
    }
}

#[path = "result_projection_tests/cases.rs"]
mod cases;
