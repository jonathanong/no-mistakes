use super::*;
use std::path::Path;

fn collect_file_facts(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> Option<TsFileFacts> {
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(&[
        path.to_path_buf(),
    ]));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    super::collect::collect_file_facts_with_sources(path, plan, context, &sources)
}

impl TsFactMap {
    pub(crate) fn extend_shared(
        &mut self,
        facts: impl IntoIterator<Item = (PathBuf, std::sync::Arc<TsFileFacts>)>,
    ) {
        self.facts.extend(
            facts
                .into_iter()
                .map(|(path, facts)| (path, std::sync::Arc::unwrap_or_clone(facts))),
        );
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-source/fixture/facts")
        .join(name)
}

#[test]
fn plan_constructors_select_expected_fact_sets() {
    let imports = TsFactPlan::imports();
    assert!(imports.imports);
    assert!(!imports.symbols);

    let both = TsFactPlan::imports_and_symbols();
    assert!(both.imports);
    assert!(both.symbols);
}

#[test]
fn source_facts_preserve_owned_public_api_and_reuse_physical_read() {
    let file = fixture("imports.ts");
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&file),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let expected = sources.read_path(&file).unwrap();

    let mut facts = collect_ts_facts_with_context_and_sources(
        std::slice::from_ref(&file),
        TsFactPlan {
            source: true,
            ..TsFactPlan::default()
        },
        &TsFactContext::default(),
        &sources,
    );

    let source: &String = facts[&file].source.as_ref().unwrap();
    assert_eq!(source, expected.as_ref());
    let symbols: Option<crate::codebase::ts_symbols::FileSymbols> = facts[&file].symbols.clone();
    assert!(symbols.is_none());
    let components: &mut Vec<crate::react_traits::report::types::ComponentFacts> =
        &mut facts.get_mut(&file).unwrap().react_components;
    components.clear();
    let owned: Vec<(PathBuf, TsFileFacts)> = facts.into_iter().collect();
    assert_eq!(owned.len(), 1);
    assert_eq!(sources.physical_read_count(), 1);
}

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
            effect_calls: true,
            ..TsFactPlan::default()
        },
        TsFactPlan {
            rsc_environment: true,
            ..TsFactPlan::default()
        },
    ] {
        assert!(plan.has_domain_facts());
    }
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
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
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

#[test]
fn queue_factory_context_without_glob_matches_all_paths() {
    let ts = fixture("imports.ts");
    let context = TsFactContext::new(ts.parent().unwrap());

    assert!(context.matches_queue_factory(&ts));
}

#[test]
fn collect_ts_facts_skips_non_indexable_files_and_preserves_read_errors() {
    let ts = fixture("imports.ts");
    let txt = fixture("plain.txt");
    let missing = fixture("missing.ts");
    let facts = collect_ts_facts(&[ts.clone(), txt, missing.clone()], TsFactPlan::imports());

    assert_eq!(facts.len(), 2);
    assert_eq!(facts[&ts].imports.len(), 1);
    assert!(facts[&ts].symbols.is_none());
    assert!(facts[&missing]
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
}

#[test]
fn collect_ts_facts_uses_tsx_parser_and_symbols_when_requested() {
    let tsx = fixture("component.tsx");
    let facts = collect_ts_facts(
        std::slice::from_ref(&tsx),
        TsFactPlan::imports_and_symbols(),
    );

    assert_eq!(facts[&tsx].imports.len(), 1);
    assert!(facts[&tsx].symbols.is_some());
}

#[test]
fn collect_ts_facts_can_skip_import_collection() {
    let ts = fixture("imports.ts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&ts),
        TsFactPlan {
            imports: false,
            symbols: false,
            ..TsFactPlan::default()
        },
    );

    assert!(facts[&ts].imports.is_empty());
    assert!(facts[&ts].symbols.is_none());
}

#[test]
fn collect_file_facts_falls_back_to_ts_source_type_for_unknown_extension() {
    let unknown = fixture("unknown-extension.source");
    let facts = collect_file_facts(&unknown, TsFactPlan::imports(), &TsFactContext::default())
        .expect("unknown extension fixture should still parse as TypeScript");

    assert_eq!(facts.imports.len(), 1);
}
