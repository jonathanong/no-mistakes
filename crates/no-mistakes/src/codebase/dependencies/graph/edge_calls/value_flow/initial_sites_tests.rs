use super::*;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};

#[cfg(feature = "test-instrumentation")]
thread_local! {
    static INITIAL_SITE_CLONES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(feature = "test-instrumentation")]
pub(super) fn record_clone() {
    INITIAL_SITE_CLONES.set(INITIAL_SITE_CLONES.get() + 1);
}

fn graph() -> DepGraph {
    let saved = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/call-site-assembly/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    DepGraph::build_with_plan(
        &root,
        &TsConfig { dir: root.clone(), paths: vec![], paths_dir: root.clone(), base_url: None },
        GraphBuildPlan { calls: true, imports: true, ..GraphBuildPlan::default() },
    ).unwrap()
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn initial_originals_clone_only_admitted_complete_occurrences() {
    let sites = graph().resolved_call_sites().to_vec();
    INITIAL_SITE_CLONES.set(0);
    let (indexes, originals) = initial_sites(&sites, [1, 5, 9].into_iter());
    assert_eq!(indexes, [1, 5, 9]);
    assert_eq!(originals, [sites[1].clone(), sites[5].clone(), sites[9].clone()]);
    assert_eq!(INITIAL_SITE_CLONES.get(), 3);
    assert_eq!(originals.capacity(), 3);
    assert!(originals.len() < sites.len());
    INITIAL_SITE_CLONES.set(0);
    let (indexes, originals) = initial_sites(&sites, std::iter::empty());
    assert!(indexes.is_empty());
    assert!(originals.is_empty());
    assert_eq!(originals.capacity(), 0);
    assert_eq!(INITIAL_SITE_CLONES.get(), 0);
}

#[test]
fn dense_work_ids_mutate_their_physical_occurrences_across_gaps() {
    let mut sites = graph().resolved_call_sites().to_vec();
    let mut expected = sites.clone();
    expected[1].source_callee.push_str(".changed");
    expected[9].source_callee.push_str(".changed");
    let selected = selected_sites(&mut sites, &[0, 2], &[1, 5, 9]);
    assert_eq!(selected.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [0, 2]);
    for (_, site) in selected { site.source_callee.push_str(".changed"); }
    assert_eq!(sites, expected);
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn known_callers_still_supply_callback_arguments_without_original_copies() {
    INITIAL_SITE_CLONES.set(0);
    let graph = graph();
    let copies = INITIAL_SITE_CLONES.get();
    let sites = graph.resolved_call_sites();
    assert!(copies > 0 && copies < sites.len(), "copies={copies}, sites={}", sites.len());
    let callback = sites.iter().find(|site| site.source_callee == "callback").unwrap();
    assert!(callback.target_nodes().any(|node| {
        matches!(node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == "sink")
    }), "known callers must remain in incoming argument facts: {callback:?}");
}

#[test]
fn parameter_dependencies_require_invocations_with_canonical_callable_ids() {
    let graph = graph();
    let mut site = graph.resolved_call_sites().iter()
        .find(|site| site.source_callee == "importedSink").unwrap().clone();
    assert_eq!(function_keys(&site).count(), 1);
    site.invocation = InvocationKind::Membership;
    assert_eq!(function_keys(&site).count(), 0);
    site.invocation = InvocationKind::Call;
    site.target_node = Some(NodeId::symbol(&site.file, "sink"));
    assert_eq!(function_keys(&site).count(), 0);
    site.target_node = Some(NodeId::file(&site.file));
    assert_eq!(function_keys(&site).count(), 0);
}
