use super::*;
use crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright;
use crate::codebase::dependencies::graph::{
    CompleteCheckFactGraphBuildRequest, DepGraph, GraphBuildPlan,
};
use crate::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use crate::codebase::ts_source::facts::TsFactPlan;

fn fixture() -> (tempfile::TempDir, NoMistakesConfig, Vec<std::path::PathBuf>) {
    let root = crate::test_support::rule_fixture_root("forbidden-calls/argument-scope");
    let saved = crate::test_support::materialize_saved_fixture(&root);
    let config = crate::config::v2::load_v2_config(
        saved.path(),
        Some(&saved.path().join(".no-mistakes.yml")),
    )
    .unwrap();
    let files = crate::codebase::ts_source::discover_files(saved.path(), &[]);
    (saved, config, files)
}

fn plan(root: &Path, config: &NoMistakesConfig) -> CheckFactPlan {
    let mut plan = CheckFactPlan {
        graph: crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(
            root,
            GraphBuildPlan {
                calls: true,
                ..Default::default()
            },
        )
        .0,
        ..Default::default()
    };
    configure_argument_facts(root, config, &mut plan);
    plan
}

#[test]
fn scoped_arguments_reduce_actual_collection_and_preserve_findings() {
    let (saved, config, files) = fixture();
    let root = saved.path();
    let tsconfig = TsConfig {
        dir: root.into(),
        paths_dir: root.into(),
        ..Default::default()
    };
    let catalog = TsConfigCatalog::forced(root, tsconfig.clone(), None);
    let mut plans = [plan(root, &config), plan(root, &config)];
    plans[1].graph.call_arguments = true;
    let mut reports = Vec::new();
    let mut graph_sites = Vec::new();
    let mut counts = Vec::new();
    for plan in plans {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
            // Keep every file graph-only to guard the secondary fact plan too.
            collect_check_facts_with_graph_files_and_playwright(
                root,
                vec![],
                files.clone(),
                plan,
                None,
            )
        });
        counts.push(observer.snapshot().work["facts.call_arguments.collections"]);
        assert!(argument_facts_covered(root, &config, &facts));
        let empty = &facts.ts.get(&root.join("src/a/empty.mjs")).unwrap().ts;
        assert!(empty
            .module_bindings
            .as_ref()
            .unwrap()
            .call_arguments
            .as_ref()
            .unwrap()
            .is_empty());
        let invalid = &facts.ts.get(&root.join("src/a/invalid.mjs")).unwrap().ts;
        assert!(invalid.parse_error.is_some());
        let graph = DepGraph::build_with_complete_check_facts_and_session(
            CompleteCheckFactGraphBuildRequest {
                root,
                tsconfig: &tsconfig,
                tsconfig_catalog: &catalog,
                plan: GraphBuildPlan {
                    calls: true,
                    ..Default::default()
                },
                files: files.clone(),
                config_path: None,
                facts: &facts,
            },
            crate::codebase::analysis_session::AnalysisSession::disabled(),
        )
        .unwrap();
        graph_sites.push(graph.resolved_call_sites().to_vec());
        reports.push(
            super::super::selection::check_with_graph(
                root,
                &config,
                &graph,
                None,
                None,
                &files,
                Some(&facts),
            )
            .unwrap(),
        );
        if !facts.graph_plan().call_arguments {
            assert!(
                !facts.graph_plan().covers(TsFactPlan {
                    call_arguments: true,
                    ..Default::default()
                }),
                "scoped facts must never advertise a global guarantee"
            );
        }
    }
    assert_eq!(
        counts,
        [4, 6],
        "target-only and excluded sources must skip argument work"
    );
    assert_eq!(reports[0], reports[1]);
    assert_eq!(graph_sites[0], graph_sites[1]);
    assert_eq!(
        reports[0].len(),
        3,
        "two argument findings and one target-only finding: {:?}",
        reports[0]
    );
}

#[test]
fn scoped_demand_merges_and_broad_graph_demand_wins() {
    let (saved, config, files) = fixture();
    let root = saved.path();
    let full = plan(root, &config);
    let scope = full.forbidden_call_arguments.as_ref().unwrap();
    let merged = ArgumentFactScope::merge(Some(scope), Some(scope)).unwrap();
    assert!(Arc::ptr_eq(scope, &merged));
    let mut first = config.clone();
    first.rules.truncate(1);
    let mut second = config.clone();
    second.rules.remove(0);
    let mut merged = plan(root, &first);
    merged.include(plan(root, &second));
    let mut facts = collect_check_facts_with_graph_files_and_playwright(
        root,
        files.clone(),
        files.clone(),
        merged,
        None,
    );
    assert!(argument_facts_covered(root, &config, &facts));
    assert!(
        argument_facts_covered(root, &first, &facts),
        "stored superset covers narrower demand"
    );
    facts.ts.remove(&root.join("src/a/empty.mjs"));
    assert!(
        !argument_facts_covered(root, &config, &facts),
        "a missing selected TS entry must reject coverage"
    );
    let facts = collect_check_facts_with_graph_files_and_playwright(
        root,
        files.clone(),
        files,
        plan(root, &first),
        None,
    );
    assert!(
        !argument_facts_covered(root, &config, &facts),
        "missing sibling demand must fail coverage"
    );
    assert!(!facts.graph_plan().call_arguments);
}

#[test]
fn absent_target_conditions_and_invalid_filters_keep_compatible_demand() {
    let (saved, mut config, _) = fixture();
    let root = saved.path();
    config.rules.drain(..2);
    assert!(for_config(root, &config).is_none());
    let mut empty = CheckFactPlan::default();
    configure_argument_facts(root, &config, &mut empty);
    assert!(empty.forbidden_call_arguments.is_none());
    assert!(ArgumentFactScope::merge(None, None).is_none());
    let (saved, mut config, _) = fixture();
    config.rules[0].include = vec!["[".into()];
    let scope = for_config(saved.path(), &config).unwrap();
    assert!(scope.applies(&saved.path().join("src/unrelated.mjs")));
    let merged = ArgumentFactScope::merge(Some(&scope), None).unwrap();
    assert!(Arc::ptr_eq(&scope, &merged));
    let other = for_config(saved.path(), &config).unwrap();
    let fallback = ArgumentFactScope::merge(Some(&scope), Some(&other)).unwrap();
    assert!(fallback.applies(&saved.path().join("src/unrelated.mjs")));
}

#[test]
fn independent_call_site_demand_survives_argument_scope() {
    let (saved, config, files) = fixture();
    let root = saved.path();
    let mut scoped = plan(root, &config);
    scoped.graph.call_sites = true;
    scoped.graph.module_bindings = true;
    let mut control = scoped.clone();
    control.forbidden_call_arguments = None;
    let scoped = collect_check_facts_with_graph_files_and_playwright(
        root,
        files.clone(),
        files.clone(),
        scoped,
        None,
    );
    let control = collect_check_facts_with_graph_files_and_playwright(
        root,
        files.clone(),
        files,
        control,
        None,
    );
    for (path, scoped) in &scoped.ts {
        let control = control.ts.get(path).unwrap();
        assert_eq!(scoped.ts.call_sites, control.ts.call_sites);
        assert_eq!(scoped.ts.function_calls, control.ts.function_calls);
        if scoped.ts.parse_error.is_none() {
            assert!(scoped.ts.module_bindings.is_some());
            assert!(control.ts.module_bindings.is_some());
        }
    }
    assert!(argument_facts_covered(
        root,
        &NoMistakesConfig::default(),
        &control
    ));
}

#[test]
fn batched_graph_only_files_keep_scoped_argument_demand() {
    use crate::codebase::check_facts::{
        collect_check_fact_batch_with_session, BatchCheckFactRequest,
    };
    let (saved, config, files) = fixture();
    let root = saved.path();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let request = BatchCheckFactRequest {
        root: root.into(),
        // Batch planning has a separate graph-only projection. No primary
        // files may accidentally carry the missing scoped argument demand.
        files: Vec::new(),
        graph_files: files,
        plan: plan(root, &config),
        playwright: None,
        sources,
    };
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        collect_check_fact_batch_with_session(&session, vec![request])
    })
    .pop()
    .unwrap();
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("facts.call_arguments.collections"),
        Some(&4)
    );
    assert!(
        argument_facts_covered(root, &config, &facts),
        "batch argument payloads must cover configured applications"
    );
    assert!(
        !facts.graph_plan().call_arguments,
        "scoped batch facts do not promise global argument coverage"
    );
    for file in ["src/a/excluded.mjs", "src/unrelated.mjs"] {
        // No independent binding consumer exists in this graph-only request;
        // omitting arguments also avoids their otherwise-unneeded binding facts.
        assert!(
            facts
                .ts
                .get(&root.join(file))
                .unwrap()
                .ts
                .module_bindings
                .is_none(),
            "{file}"
        );
    }
}

#[test]
fn argument_scope_alone_requests_parsing_only_for_selected_sources() {
    let (saved, config, files) = fixture();
    let root = saved.path();
    let mut scoped = CheckFactPlan {
        // A raw-source-only consumer disables the default AST compatibility
        // fallback, leaving scoped arguments as the only AST demand.
        raw_source: true,
        ..Default::default()
    };
    configure_argument_facts(root, &config, &mut scoped);
    assert!(scoped.graph.is_empty());
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        crate::codebase::check_facts::collect_check_facts(root, files, scoped)
    });
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("facts.call_arguments.collections"),
        Some(&4)
    );
    for (path, file_facts) in &facts.ts {
        assert!(file_facts.source.is_some(), "{}", path.display());
    }
    for file in [
        "src/a/entry.mjs",
        "src/a/helper.mjs",
        "src/a/empty.mjs",
        "src/b/entry.mjs",
    ] {
        assert!(
            facts
                .ts
                .get(&root.join(file))
                .unwrap()
                .ts
                .module_bindings
                .as_ref()
                .unwrap()
                .call_arguments
                .is_some(),
            "{file}"
        );
    }
    for file in [
        "src/a/entry.mjs",
        "src/a/helper.mjs",
        "src/a/empty.mjs",
        "src/a/invalid.mjs",
        "src/b/entry.mjs",
    ] {
        assert!(facts.ts.get(&root.join(file)).unwrap().parsed, "{file}");
    }
    for file in ["src/a/excluded.mjs", "src/unrelated.mjs"] {
        let file_facts = facts.ts.get(&root.join(file)).unwrap();
        assert!(!file_facts.parsed, "{file}");
        assert!(file_facts.ts.module_bindings.is_none(), "{file}");
    }
    assert!(argument_facts_covered(root, &config, &facts));
    assert!(!facts.graph_plan().call_arguments);
}
