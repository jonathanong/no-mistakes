use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use std::path::Path;

fn fixture() -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/owned-batch-parse-cache/fixture"),
    )
}

fn requests(session: &AnalysisSession, root: &Path, path: &Path) -> Vec<BatchCheckFactRequest> {
    let sources = session.dataset(root).sources_for(root);
    [false, true]
        .into_iter()
        .map(|symbols| BatchCheckFactRequest {
            root: root.to_path_buf(),
            files: vec![path.to_path_buf()],
            graph_files: Vec::new(),
            plan: CheckFactPlan {
                imports: true,
                symbols,
                ..Default::default()
            },
            playwright: None,
            sources: sources.clone(),
        })
        .collect()
}

fn demands(requests: &[BatchCheckFactRequest]) -> Vec<FactDemand> {
    requests
        .iter()
        .enumerate()
        .map(|(request, input)| FactDemand {
            request,
            plan: input.plan.clone(),
        })
        .collect()
}

fn warm_ambient(session: &AnalysisSession, root: &Path, path: &Path) {
    let source = session.read_source(&root.join("ambient.ts")).unwrap();
    session.with_program(path, &source, |_, _| ()).unwrap();
    assert_eq!(crate::ast::request_parse_cache_len(), 1);
}

fn assert_ambient_restored(session: &AnalysisSession, path: &Path) {
    assert_eq!(crate::ast::request_parse_cache_len(), 1);
    let current = session.read_source(path).unwrap();
    let retained = session
        .with_program(path, &current, |_, source| source.to_owned())
        .unwrap();
    assert!(
        retained.contains("export function ambientValue"),
        "{retained}"
    );
}

fn assert_current(facts: &super::super::CheckFileFacts) {
    assert!(
        facts
            .ts
            .exported_bindings
            .iter()
            .any(|binding| binding.exported == "currentValue"),
        "worker facts must come from its request, not the ambient cache: {:?}",
        facts.ts.exported_bindings
    );
    assert!(!facts
        .ts
        .exported_bindings
        .iter()
        .any(|binding| binding.exported == "ambientValue"));
}

#[test]
fn rayon_worker_owns_cache_even_when_another_request_cache_is_active() {
    let fixture = fixture();
    let root = normalize(fixture.path());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        crate::ast::with_request_parse_cache(|| {
            let path = root.join("current.ts");
            let outer = AnalysisSession::disabled();
            warm_ambient(&outer, &root, &path);
            let session = AnalysisSession::disabled();
            let requests = requests(&session, &root, &path);
            crate::ast::begin_parse_count_this_thread(&root);
            // The coordinator has already queued this demand. A stolen Rayon task
            // may begin with a foreign same-path program active on its worker.
            let (_, _, facts) =
                collect_demand(&session, &requests, path.clone(), demands(&requests), true)
                    .unwrap();
            let counts = crate::ast::finish_parse_count(&root);
            assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
            assert_eq!(facts.len(), 2);
            for facts in &facts {
                assert_current(facts.as_ref().unwrap());
            }
            assert!(facts[1].as_ref().unwrap().symbols.is_some());
            assert_eq!(requests[0].sources.physical_read_count(), 1);
            assert_ambient_restored(&outer, &path);
        })
    });
}

#[test]
fn cached_coordinator_projects_shared_variants_without_reparsing() {
    let fixture = fixture();
    let root = normalize(fixture.path());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        crate::ast::with_request_parse_cache(|| {
            let session = AnalysisSession::disabled();
            let path = root.join("current.ts");
            let source = session.read_source(&path).unwrap();
            crate::ast::begin_parse_count_this_thread(&root);
            session.with_program(&path, &source, |_, _| ()).unwrap();
            let facts =
                collect_check_fact_batch_with_session(&session, requests(&session, &root, &path));
            let counts = crate::ast::finish_parse_count(&root);
            assert_eq!(
                counts.get(&path),
                Some(&1),
                "prepared coordinator AST must be reused: {counts:#?}"
            );
            assert_eq!(facts.len(), 2);
            for facts in &facts {
                assert_current(&facts.ts[&path]);
            }
            assert!(facts[1].ts[&path].symbols.is_some());
            assert_eq!(crate::ast::request_parse_cache_len(), 1);
        })
    });
}

#[test]
fn worker_failures_restore_the_ambient_parse_cache() {
    let fixture = fixture();
    let root = normalize(fixture.path());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        crate::ast::with_request_parse_cache(|| {
            let session = AnalysisSession::disabled();
            let path = root.join("current.ts");
            warm_ambient(&session, &root, &path);
            let missing = root.join("missing.ts");
            let requests = requests(&session, &root, &missing);
            let (_, _, facts) =
                collect_demand(&session, &requests, missing, demands(&requests), true).unwrap();
            assert!(facts.iter().all(|facts| facts
                .as_ref()
                .unwrap()
                .ts
                .operational_error
                .is_some()));
            assert_ambient_restored(&session, &path);
            let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // An invalid internal demand panics after entering the worker scope.
                collect_demand(
                    &session,
                    &requests,
                    path.clone(),
                    vec![FactDemand {
                        request: usize::MAX,
                        plan: CheckFactPlan::default(),
                    }],
                    true,
                );
            }));
            assert!(panic.is_err());
            assert_ambient_restored(&session, &path);
        })
    });
}

mod retention;
