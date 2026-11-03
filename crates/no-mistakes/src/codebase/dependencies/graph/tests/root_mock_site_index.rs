use super::super::{
    DepGraph, GraphBuildPlan, GraphFiles, MockSiteIndex, MockedModule, ResolvedCallSite,
    ResolvedCallTarget, TsFactLookup,
};
use crate::codebase::dependencies::extract::{ExportedBinding, InvocationKind};
use crate::codebase::ts_resolver::{
    normalize_path, ImportClassification, ImportResolution, VisiblePathLookup,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::{fx_map, FxHashMap, FxHashSet};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingResolver {
    calls: AtomicUsize,
}

impl ImportResolution for CountingResolver {
    fn resolve(&self, specifier: &str, _: &Path) -> Option<PathBuf> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        specifier.strip_prefix("./").map(path)
    }

    fn resolution_candidates(&self, _: &str, _: &Path) -> BTreeSet<PathBuf> {
        BTreeSet::new()
    }

    fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        None
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &IndexedWorkspaceMap,
        _: &dyn VisiblePathLookup,
    ) -> ImportClassification {
        ImportClassification::unresolved()
    }

    fn matches_paths_alias(&self, _: &str, _: &Path) -> bool {
        false
    }
}

struct Facts(FxHashMap<PathBuf, TsFileFacts>);

impl TsFactLookup for Facts {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.0.get(path)
    }
}

fn path(name: &str) -> PathBuf {
    normalize_path(&PathBuf::from("/tmp/nm-site-index").join(name))
}

fn facts(exports: &[(&str, &str)], stars: &[&str]) -> TsFileFacts {
    TsFileFacts {
        exported_bindings: exports
            .iter()
            .map(|(specifier, name)| ExportedBinding {
                specifier: Some((*specifier).into()),
                local: (*name).into(),
                exported: (*name).into(),
            })
            .collect(),
        star_reexport_specifiers: stars.iter().map(|star| (*star).into()).collect(),
        ..TsFileFacts::default()
    }
}

fn site(file: &str, specifier: &str, offset: u32) -> ResolvedCallSite {
    ResolvedCallSite {
        file: path(file),
        caller: None,
        caller_id: None,
        line: 1,
        offset,
        invocation: InvocationKind::Call,
        source_callee: "run".into(),
        target: ResolvedCallTarget::ModuleExport {
            specifier: specifier.into(),
            export_path: "run".into(),
            repository_target: None,
            callable_id: None,
        },
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

fn modules(names: &[&str]) -> FxHashSet<MockedModule> {
    names
        .iter()
        .map(|name| MockedModule::File(path(name)))
        .collect()
}

fn graph() -> DepGraph {
    let root = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/production-exports-only-used-by-tests/fixture/class-extends-production",
    ));
    let mut graph = DepGraph::build_with_plan(
        &root,
        &crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..crate::codebase::ts_resolver::TsConfig::default()
        },
        GraphBuildPlan::default(),
    )
    .unwrap();
    graph.resolved_call_sites = vec![
        site("via-next.mts", "./barrel.mts", 1),
        site("via-stars.mts", "./stars.mts", 2),
        site("via-miss.mts", "./half.mts", 3),
    ];
    graph
}

fn fact_map() -> Facts {
    let mut map: FxHashMap<PathBuf, TsFileFacts> = fx_map();
    map.insert(path("barrel.mts"), facts(&[("./impl.mts", "run")], &[]));
    map.insert(
        path("stars.mts"),
        facts(&[], &["./left.mts", "./right.mts"]),
    );
    map.insert(path("half.mts"), facts(&[], &["./left.mts", "./empty.mts"]));
    map.insert(path("left.mts"), facts(&[("./l.mts", "run")], &[]));
    map.insert(path("right.mts"), facts(&[("./r.mts", "run")], &[]));
    map.insert(path("empty.mts"), facts(&[], &[]));
    Facts(map)
}

fn suppressed(
    graph: &DepGraph,
    root: &str,
    mocked: &[&str],
    context: (&Facts, &CountingResolver, &GraphFiles, &MockSiteIndex),
) -> BTreeSet<u32> {
    let (facts, resolver, files, index) = context;
    let found = graph.calls_through_mocked_modules(
        &path(root),
        &modules(mocked),
        facts,
        resolver,
        files,
        index,
    );
    found
        .into_iter()
        .map(|(_, offset)| offset)
        .collect::<BTreeSet<_>>()
}

#[test]
fn indexed_sites_keep_next_and_star_suppression_semantics() {
    let graph = graph();
    let facts = fact_map();
    let resolver = CountingResolver {
        calls: AtomicUsize::new(0),
    };
    let files = GraphFiles::from_files(vec![path("a.mts")]);
    let index = graph.build_mock_site_index(&facts, &resolver, &files);
    let context = (&facts, &resolver, &files, &index);
    assert_eq!(
        suppressed(&graph, "root-a.mts", &["impl.mts"], context),
        BTreeSet::from([1])
    );
    assert_eq!(
        suppressed(&graph, "root-a.mts", &["l.mts", "r.mts"], context),
        BTreeSet::from([2])
    );
    assert_eq!(
        suppressed(&graph, "root-a.mts", &["l.mts"], context),
        BTreeSet::new()
    );
    assert_eq!(
        suppressed(&graph, "via-next.mts", &["impl.mts"], context),
        BTreeSet::new()
    );
}

#[test]
fn resolver_calls_do_not_scale_with_test_roots() {
    let graph = graph();
    let facts = fact_map();
    let resolver = CountingResolver {
        calls: AtomicUsize::new(0),
    };
    let files = GraphFiles::from_files(vec![path("a.mts")]);
    let index = graph.build_mock_site_index(&facts, &resolver, &files);
    let context = (&facts, &resolver, &files, &index);
    suppressed(&graph, "root-a.mts", &["impl.mts"], context);
    let after_one = resolver.calls.load(Ordering::SeqCst);
    assert!(after_one > 0);
    for root in ["root-b.mts", "root-c.mts", "root-d.mts"] {
        suppressed(&graph, root, &["impl.mts", "l.mts", "r.mts"], context);
    }
    assert_eq!(resolver.calls.load(Ordering::SeqCst), after_one);
}
