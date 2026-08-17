use super::*;

const EXPECTED_FILES: usize = 53;
const EXPECTED_PARSED: usize = 39;
const EXPECTED_IMPORTS: usize = 49;
const EXPECTED_ENQUEUES: usize = 7;
const EXPECTED_WORKERS: usize = 6;
const EXPECTED_ROUTES: usize = 12;
const EXPECTED_EDGES: usize = 68;
const EXPECTED_QUEUE_EDGES: usize = 14;
const EXPECTED_GLOB_MATCHES: usize = 53;

#[test]
fn language_frontend_adapters_drive_production_collectors() {
    let fixture = language_frontend_fixture();
    let facts = collect_language_frontend_facts(&fixture);
    let edges = collect_language_frontend_edges(&fixture);
    let globs = match_language_frontend_queue_globs(&fixture);
    assert_eq!(
        facts,
        LanguageFrontendSummary {
            files: EXPECTED_FILES,
            parsed_files: EXPECTED_PARSED,
            imports: EXPECTED_IMPORTS,
            enqueues: EXPECTED_ENQUEUES,
            workers: EXPECTED_WORKERS,
            route_handlers: EXPECTED_ROUTES,
            ..LanguageFrontendSummary::default()
        }
    );
    assert_eq!(
        edges,
        LanguageFrontendSummary {
            files: EXPECTED_FILES,
            edges: EXPECTED_EDGES,
            queue_edges: EXPECTED_QUEUE_EDGES,
            ..LanguageFrontendSummary::default()
        }
    );
    assert_eq!(
        globs,
        LanguageFrontendSummary {
            files: EXPECTED_FILES,
            glob_matches: EXPECTED_GLOB_MATCHES,
            ..LanguageFrontendSummary::default()
        }
    );
    assert_eq!(
        collect_language_frontend_facts(&fixture),
        facts,
        "composed fixture collection must be deterministic"
    );
}

#[test]
fn queue_glob_match_evaluates_enqueue_and_worker_lists_independently() {
    let fixture = language_frontend_fixture();
    let enqueue_only = crate::codebase::dependencies::graph::count_queue_glob_matches(
        &fixture.root,
        &fixture.files,
        &["kafka-topics/**".into()],
        &[],
    );
    let worker_only = crate::codebase::dependencies::graph::count_queue_glob_matches(
        &fixture.root,
        &fixture.files,
        &[],
        &["python-celery-django/**".into()],
    );
    let both = crate::codebase::dependencies::graph::count_queue_glob_matches(
        &fixture.root,
        &fixture.files,
        &["kafka-topics/**".into()],
        &["python-celery-django/**".into()],
    );
    assert!(
        enqueue_only > 0,
        "enqueue globs must match kafka-topics files"
    );
    assert!(
        worker_only > 0,
        "worker globs must match python-celery-django files"
    );
    assert_eq!(
        both,
        enqueue_only + worker_only,
        "independent list evaluation must union disjoint enqueue and worker matches"
    );
}
