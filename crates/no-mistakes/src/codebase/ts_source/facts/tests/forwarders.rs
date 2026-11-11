use super::*;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, GraphFiles};
use crate::codebase::ts_resolver::TsConfig;

#[test]
fn imported_forwarder_identity_is_independent_of_extra_fact_demands() {
    for value_flow in [false, true] {
        for (name, transparent) in [
            ("captured-wrapper-owned", true),
            ("captured-wrapper-inspected", true),
            ("captured-wrapper-mutated", false),
            ("captured-wrapper-replaced", false),
            ("captured-wrapper-effect-call", false),
            ("captured-wrapper-opaque-constructor", false),
            ("captured-wrapper-promises-owned", true),
            ("captured-wrapper-typed-owned", true),
            ("captured-wrapper-aliased-write", false),
        ] {
            let path = crate::codebase::ts_resolver::normalize_path(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../test-cases/rules/test-no-live-repository-scan/env-provenance")
                    .join(name)
                    .join("root.mts"),
            );
            let context = TsFactContext::new(path.parent().unwrap());
            let base = TsFactPlan {
                function_calls: true,
                value_flow,
                ..Default::default()
            };
            let ordinary = collect_file_facts(&path, base, &context).unwrap();
            let enriched = collect_file_facts(
                &path,
                TsFactPlan {
                    module_bindings: true,
                    call_arguments: true,
                    ..base
                },
                &context,
            )
            .unwrap();
            assert_eq!(ordinary.function_calls, enriched.function_calls, "{name}");
            assert_eq!(
                ordinary.callable_aliases, enriched.callable_aliases,
                "{name}"
            );
            assert_eq!(
                enriched
                    .module_bindings
                    .as_ref()
                    .unwrap()
                    .transparent_receiver_writes
                    .len(),
                if transparent { 2 } else { 0 },
                "{name}"
            );
            if value_flow {
                let ordinary = resolved_calls(&path, ordinary, base);
                let enriched = resolved_calls(
                    &path,
                    enriched,
                    TsFactPlan {
                        module_bindings: true,
                        call_arguments: true,
                        ..base
                    },
                );
                assert_eq!(ordinary, enriched, "{name}");
                let call = ordinary
                    .iter()
                    .find(|call| {
                        call.source_callee == "fs.readdir"
                            && call.caller.as_deref() == Some("withFixture")
                    })
                    .unwrap();
                assert_eq!(!call.has_unknown_target(), transparent, "{name}: {call:?}");
            }
        }
    }
}

fn resolved_calls(
    path: &Path,
    facts: TsFileFacts,
    plan: TsFactPlan,
) -> Vec<crate::codebase::dependencies::graph::ResolvedCallSite> {
    let root = path.parent().unwrap();
    let config = TsConfig {
        dir: root.to_path_buf(),
        paths: vec![],
        paths_dir: root.to_path_buf(),
        base_url: None,
    };
    let files = GraphFiles::discover(root);
    let facts = TsFactMap::from_iter_with_plan([(path.to_path_buf(), facts)], plan);
    let graph = DepGraph::build_with_plan_files_config_and_facts(
        root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
        &files,
        None,
        Some(&facts),
    )
    .unwrap();
    graph.resolved_call_sites().to_vec()
}

#[test]
fn plan_coverage_tracks_call_site_facts() {
    let available = TsFactPlan {
        call_sites: true,
        ..TsFactPlan::default()
    };
    assert!(available.covers(TsFactPlan {
        call_sites: true,
        ..TsFactPlan::default()
    }));
    assert!(!TsFactPlan::default().covers(TsFactPlan {
        call_sites: true,
        ..TsFactPlan::default()
    }));
}

#[test]
fn per_item_demand_survives_plan_union_and_requires_complete_facts() {
    let required = TsFactPlan {
        function_calls: true,
        per_item_calls: true,
        ..TsFactPlan::default()
    };
    let mut shared = TsFactPlan::imports();
    // Ordinary call facts cannot answer an iteration policy without its demand.
    assert!(!shared.covers(required));
    shared.include(required);
    assert!(shared.covers(required));
    assert!(!shared.is_empty());
}
