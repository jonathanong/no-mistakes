use super::{collect_check_fact_batch_with_session, BatchCheckFactRequest};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::ts_source::facts::TsFactPlan;

#[test]
fn batch_workers_do_not_retain_programs_in_an_outer_worker_cache() {
    let saved = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-facts/bounded-session-memory");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let session = AnalysisSession::new(None);
    let sources = session.dataset(&root).sources_for(&root);
    let outer = root.join("outer.ts");
    let source = root.join("source.ts");
    let request = BatchCheckFactRequest {
        root: root.clone(),
        files: vec![source.clone()],
        graph_files: Vec::new(),
        plan: CheckFactPlan {
            graph: TsFactPlan::imports_and_symbols(),
            ..Default::default()
        },
        playwright: None,
        sources,
    };
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    crate::ast::begin_parse_count(&root);
    pool.install(|| {
        crate::ast::with_owned_request_parse_cache(|| {
            let text = session.read_source(&outer).unwrap();
            session.with_program(&outer, &text, |_, _| ()).unwrap();
            assert_eq!(crate::ast::request_parse_cache_len(), 1);
            let facts = collect_check_fact_batch_with_session(&session, vec![request]);
            let file = &facts[0].ts[&source];
            assert_eq!(file.ts.imports.len(), 1);
            assert!(!file.ts.exported_bindings.is_empty());
            // A nested task must release its AST and restore its caller's cache.
            assert_eq!(crate::ast::request_parse_cache_len(), 1);
        });
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&outer), Some(&1), "{counts:#?}");
    assert_eq!(counts.get(&source), Some(&1), "{counts:#?}");
}
