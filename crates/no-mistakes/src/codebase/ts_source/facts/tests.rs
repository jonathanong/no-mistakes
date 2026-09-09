use super::*;
use std::path::{Path, PathBuf};

mod map;

fn collect_file_facts(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> Option<TsFileFacts> {
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(&[
        path.to_path_buf(),
    ]));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    super::collect::test_support::collect_file_facts_with_sources(path, plan, context, &sources)
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-source/fixture/facts")
        .join(name)
}

include!("tests/collection.rs");
include!("tests/session_reuse.rs");

#[test]
fn pass4b_react_graph_facts_skip_ignored_child_for_visible_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4a-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let mut context = TsFactContext::new(&root);
    context.set_visible_files(visible_paths.iter().cloned());
    let parent = root.join("react/Parent.tsx");

    let facts = collect_ts_facts_with_context(
        std::slice::from_ref(&parent),
        TsFactPlan {
            react: true,
            ..TsFactPlan::default()
        },
        &context,
    );

    assert_eq!(
        facts[&parent].react_components[0].children[0].file,
        "react/Child.ts"
    );
}

#[test]
fn react_graph_facts_support_an_unscoped_context() {
    let component = fixture("component.tsx");
    let context = TsFactContext::new(component.parent().unwrap());

    let facts = collect_ts_facts_with_context(
        std::slice::from_ref(&component),
        TsFactPlan {
            react: true,
            ..TsFactPlan::default()
        },
        &context,
    );

    assert_eq!(facts[&component].react_components.len(), 1);
}

#[test]
fn fact_context_include_merges_backend_route_extractors() {
    let root = fixture("");
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("**/*.ts").unwrap());
    let mut added = TsFactContext::new(&root);
    added.add_backend_route_extractor(
        "router".to_string(),
        "get($ROUTE, $HANDLER)".to_string(),
        builder.build().unwrap(),
    );
    let mut context = TsFactContext::new(&root);

    context.include(added);

    assert_eq!(context.backend_route_extractors.len(), 1);
    assert_eq!(
        context.backend_route_extractors[0].register_object,
        "router"
    );
}

#[test]
fn fact_context_include_merges_server_route_filter() {
    let root = fixture("");
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("routes/**").unwrap());
    let mut added = TsFactContext::new(&root);
    added.set_server_route_filter(
        builder.build().unwrap(),
        Some(crate::codebase::test_filter::TestFileFilter::fallback_only()),
    );
    assert!(format!("{added:?}").contains("ServerRouteFactFilter"));

    let mut context = TsFactContext::new(&root);
    context.include(added);

    assert!(context.matches_server_route(&root.join("routes/users.ts")));
    assert!(!context.matches_server_route(&root.join("client.ts")));
    assert!(!context.matches_server_route(&root.join("routes/users.test.ts")));
}

#[test]
fn plan_domain_fact_detection_tracks_domain_flags() {
    assert!(!TsFactPlan::default().has_domain_facts());
    assert!(!TsFactPlan {
        imports: true,
        symbols: true,
        ..TsFactPlan::default()
    }
    .has_domain_facts());
    assert!(!TsFactPlan {
        source: true,
        ..TsFactPlan::default()
    }
    .has_domain_facts());

    for plan in [
        TsFactPlan {
            route_refs: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            backend_routes: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            queue_usage: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            queue_factory: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            http_calls: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            process_spawns: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            rsc_environment: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            trpc_router: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            trpc_calls: true,
            ..TsFactPlan::default()
        },
    ] {
        assert!(plan.has_domain_facts());
    }
    assert!(!TsFactPlan {
        effect_calls: true,
        ..TsFactPlan::default()
    }
    .has_domain_facts());
}

#[test]
fn effects_can_use_the_plain_canonical_fact_collection() {
    let file = fixture("imports.ts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            effect_calls: true,
            ..TsFactPlan::default()
        },
    );

    assert!(!facts[&file].function_calls.is_empty());
    assert!(facts[&file].effect_calls.is_empty());
}

#[test]
fn effect_projection_does_not_start_a_domain_ast_walk() {
    let file = fixture("imports.ts");
    let mut context = TsFactContext::new(file.parent().unwrap());
    context.effect_functions.insert("helper".to_string(), None);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let facts = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        collect_file_facts(
            &file,
            TsFactPlan {
                effect_calls: true,
                ..TsFactPlan::default()
            },
            &context,
        )
        .unwrap()
    };

    let helper_calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "helper")
        .collect();
    assert_eq!(helper_calls.len(), 2, "{helper_calls:#?}");
    assert_eq!(helper_calls[0].offset, helper_calls[1].offset);
    assert!(helper_calls.iter().any(|call| call.caller.is_some()));
    assert!(helper_calls.iter().any(|call| call.caller.is_none()));
    assert_eq!(facts.effect_calls.len(), 1, "{facts:#?}");
    assert_eq!(facts.effect_calls[0].caller, None);
    assert!(
        !observer.snapshot().work.contains_key("ast.walks"),
        "effects must project canonical calls without a second AST visitor: {:#?}",
        observer.snapshot()
    );
}

#[test]
fn effect_projection_keeps_distinct_same_line_calls() {
    let file = fixture("same-line-effects.ts");
    let mut context = TsFactContext::new(file.parent().unwrap());
    context.effect_functions.insert("helper".to_string(), None);

    let facts = collect_file_facts(
        &file,
        TsFactPlan {
            effect_calls: true,
            ..TsFactPlan::default()
        },
        &context,
    )
    .unwrap();

    assert_eq!(
        facts.effect_calls.len(),
        2,
        "one same-line call is inside the exported function and one is a distinct top-level call: {:#?}",
        facts.effect_calls
    );
    assert_eq!(
        facts
            .effect_calls
            .iter()
            .filter(|call| call.caller.as_deref() == Some("value"))
            .count(),
        1
    );
    assert_eq!(
        facts
            .effect_calls
            .iter()
            .filter(|call| call.caller.is_none())
            .count(),
        1
    );
}

#[test]
#[should_panic(expected = "domain fact plans require collect_ts_facts_with_context")]
fn collect_ts_facts_rejects_context_required_domain_plans() {
    let ts = fixture("imports.ts");
    let _facts = collect_ts_facts(
        std::slice::from_ref(&ts),
        TsFactPlan {
            http_calls: true,
            ..TsFactPlan::default()
        },
    );
}

#[test]
fn collect_ts_facts_can_include_source_without_domain_context() {
    let ts = fixture("imports.ts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&ts),
        TsFactPlan {
            source: true,
            ..TsFactPlan::default()
        },
    );

    assert!(facts[&ts]
        .source
        .as_deref()
        .unwrap_or("")
        .contains("import"));
}

#[test]
fn collected_fact_map_retains_its_plan_and_read_errors() {
    let missing = fixture("missing.ts");
    let plan = TsFactPlan::imports();
    let facts = collect_ts_facts(std::slice::from_ref(&missing), plan);

    assert!(facts.plan().covers(plan));
    assert!(facts[&missing]
        .operational_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
    assert!(facts[&missing]
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
}

#[test]
fn failed_collection_result_becomes_operational_error_facts() {
    let facts = super::collect::test_support::facts_from_collection_result(Err(anyhow::anyhow!(
        "synthetic parse failure"
    )));

    assert_eq!(
        facts.operational_error.as_deref(),
        Some("synthetic parse failure")
    );
    assert_eq!(
        facts.parse_error.as_deref(),
        Some("synthetic parse failure")
    );
    assert!(!facts.fatal_parse_error);
    assert!(facts.symbols.is_none());
}

#[test]
fn fact_map_supports_hash_map_compatible_iteration() {
    let path = fixture("imports.ts");
    let mut facts = TsFactMap::new();
    facts.insert(path.clone(), TsFileFacts::default());

    assert_eq!((&facts).into_iter().count(), 1);
    for (_, file_facts) in &mut facts {
        file_facts.source = Some("updated".into());
    }

    let entries = facts.into_iter().collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, path);
    assert_eq!(entries[0].1.source.as_deref(), Some("updated"));
}

#[test]
fn plan_empty_detection_tracks_all_flags() {
    assert!(TsFactPlan::default().is_empty());

    for plan in [
        TsFactPlan {
            imports: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            symbols: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            source: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            call_sites: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            route_refs: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            backend_routes: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            queue_usage: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            queue_factory: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            http_calls: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            process_spawns: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            effect_calls: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            rsc_environment: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            trpc_router: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            trpc_calls: true,
            ..TsFactPlan::default()
        },
    ] {
        assert!(!plan.is_empty());
    }
}

#[test]
fn plan_coverage_tracks_effect_and_rsc_facts() {
    let available = TsFactPlan {
        effect_calls: true,
        rsc_environment: true,
        ..TsFactPlan::default()
    };

    assert!(available.covers(TsFactPlan {
        effect_calls: true,
        ..TsFactPlan::default()
    }));
    assert!(available.covers(TsFactPlan {
        rsc_environment: true,
        ..TsFactPlan::default()
    }));
    assert!(!TsFactPlan::default().covers(TsFactPlan {
        effect_calls: true,
        ..TsFactPlan::default()
    }));
    assert!(!TsFactPlan::default().covers(TsFactPlan {
        rsc_environment: true,
        ..TsFactPlan::default()
    }));
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
fn unscoped_domain_fact_context_does_not_collect_config_scoped_facts() {
    let ts = fixture("imports.ts");
    let context = TsFactContext::new(ts.parent().unwrap());
    let facts = collect_ts_facts_with_context(
        std::slice::from_ref(&ts),
        TsFactPlan {
            backend_routes: true,
            queue_factory: true,
            ..TsFactPlan::default()
        },
        &context,
    );
    let file_facts = &facts[&ts];

    assert!(file_facts.backend_routes.is_empty());
    assert!(file_facts.queue_create_line.is_none());
    assert!(file_facts.queue_name.is_none());
}

#[test]
fn queue_factory_context_requires_specifier_and_function_even_when_glob_matches() {
    let ts = fixture("imports.ts");
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("*.ts").unwrap());
    let mut context = TsFactContext::new(ts.parent().unwrap());
    context.queue_factory_glob = Some(builder.build().unwrap());
    let facts = collect_ts_facts_with_context(
        std::slice::from_ref(&ts),
        TsFactPlan {
            queue_factory: true,
            ..TsFactPlan::default()
        },
        &context,
    );

    assert!(facts[&ts].queue_create_line.is_none());
    assert!(facts[&ts].queue_name.is_none());
}

#[path = "tests/collection_regressions.rs"]
mod collection_regressions;
