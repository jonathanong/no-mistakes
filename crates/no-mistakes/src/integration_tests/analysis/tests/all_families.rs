use super::*;
use crate::codebase::ts_source::facts::TsFactPlan;

#[test]
fn all_ts_and_primary_check_families_keep_full_integration_findings() {
    let root = fixture();
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let mut plan = CheckFactPlan {
        imports: true,
        symbols: true,
        react: true,
        react_usages: true,
        queue: true,
        integration: true,
        dynamic_imports: true,
        nextjs_caching: true,
        nextjs_caching_low: true,
        storybook: true,
        server_route_client_boundary: true,
        source: true,
        raw_source: true,
        embedded_sql: true,
        query_annotation: true,
        query_annotation_heads: true,
        graph: TsFactPlan {
            module_bindings: true,
            call_arguments: true,
            call_eligibility: true,
            imports: true,
            function_calls: true,
            value_flow: true,
            identifier_provenance: true,
            per_item_calls: true,
            call_sites: true,
            resources: true,
            symbols: true,
            source: true,
            route_refs: true,
            backend_routes: true,
            queue_usage: true,
            queue_factory: true,
            queue_project: true,
            http_calls: true,
            process_spawns: true,
            server_routes: true,
            react: true,
            effect_calls: true,
            rsc_environment: true,
            trpc_router: true,
            trpc_calls: true,
        },
        ..Default::default()
    };
    let full = collect_check_facts(&root, files.clone(), plan.clone());
    configure_family_path_scopes(&root, &files, &config, &sources, &mut plan).unwrap();
    let scoped = collect_check_facts(&root, files, plan);
    let report = |facts: &crate::codebase::check_facts::CheckFactMap| {
        let analyses = facts
            .ts
            .iter()
            .filter_map(|(path, facts)| facts.integration.clone().map(|item| (path.clone(), item)))
            .collect();
        checks::check_suites(
            &root,
            &test_support::configured_suites(&root, &config).unwrap(),
            &test_support::tsconfig_without_config(&root),
            &analyses,
            &crate::codebase::analysis_session::AnalysisSession::disabled(),
            facts,
        )
        .unwrap()
    };
    assert_eq!(report(&scoped), report(&full));
    assert_eq!(report(&scoped).len(), 7);
    assert_eq!(scoped.stats.files_parsed, full.stats.files_parsed);
    for (path, scoped_file) in &scoped.ts {
        let full_file = &full.ts[path];
        assert_eq!(scoped_file.ts.imports, full_file.ts.imports, "{path:?}");
        assert_eq!(
            scoped_file.ts.function_calls, full_file.ts.function_calls,
            "{path:?}"
        );
        assert_eq!(scoped_file.ts.source, full_file.ts.source, "{path:?}");
        assert_eq!(scoped_file.parse_error, full_file.parse_error, "{path:?}");
    }
}
