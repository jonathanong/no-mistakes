use super::*;

#[test]
fn prepared_config_programs_share_the_union_fact_pass_with_uncached_sources() {
    let saved =
        crate::test_support::rule_fixture_root("vitest-project-policy").join("shared-prepared");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let files = ["vitest.config.ts", "helper.ts", "unit.test.ts"].map(|name| root.join(name));
    let collect = |prewarm: bool| {
        let session = crate::codebase::analysis_session::AnalysisSession::disabled();
        let sources = Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&files),
        )));
        crate::ast::begin_parse_count(&root);
        let facts = crate::ast::with_owned_request_parse_cache(|| {
            if prewarm {
                for path in &files[..2] {
                    let source = sources.read_path(path).unwrap();
                    crate::ast::with_program(path, &source, |_, _| ()).unwrap();
                }
            }
            super::super::collect_check_facts_with_graph_files_playwright_sources_and_session(
                &session,
                &root,
                (files.to_vec(), Vec::new()),
                CheckFactPlan {
                    imports: true,
                    graph: crate::codebase::ts_source::facts::TsFactPlan {
                        imports: true,
                        call_sites: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                sources,
            )
        });
        let counts = crate::ast::finish_parse_count(&root);
        for path in &files {
            assert_eq!(counts.get(path), Some(&1), "prewarm={prewarm}: {counts:#?}");
        }
        files
            .iter()
            .map(|path| {
                let file = facts.ts.get(path).unwrap();
                assert!(file.parsed);
                assert!(file.parse_error.is_none());
                (file.ts.imports.clone(), file.ts.call_sites.clone())
            })
            .collect::<Vec<_>>()
    };
    let cold = collect(false);
    let prepared = collect(true);
    assert_eq!(prepared, cold);
    assert!(!prepared[0].0.is_empty());
    assert!(prepared[2].1.iter().any(|call| call.callee == "setTimeout"));
}
