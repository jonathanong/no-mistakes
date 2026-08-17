use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{
    collect_language_frontend_edges, collect_language_frontend_facts, language_frontend_fixture,
    match_language_frontend_queue_globs, LanguageFrontendSummary,
};

pub(super) const EXPECTED_LANGUAGE_FRONTEND_FILES: usize = 61;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_PARSED: usize = 39;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_IMPORTS: usize = 49;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_ENQUEUES: usize = 7;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_WORKERS: usize = 6;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_ROUTES: usize = 12;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_EDGES: usize = 68;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_QUEUE_EDGES: usize = 14;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_GLOB_MATCHES: usize = 61;

pub(super) fn bench_language_frontends(c: &mut Criterion) {
    let fixture = language_frontend_fixture();
    let facts = collect_language_frontend_facts(&fixture);
    let edges = collect_language_frontend_edges(&fixture);
    let globs = match_language_frontend_queue_globs(&fixture);
    assert_eq!(
        facts,
        LanguageFrontendSummary {
            files: EXPECTED_LANGUAGE_FRONTEND_FILES,
            parsed_files: EXPECTED_LANGUAGE_FRONTEND_PARSED,
            imports: EXPECTED_LANGUAGE_FRONTEND_IMPORTS,
            enqueues: EXPECTED_LANGUAGE_FRONTEND_ENQUEUES,
            workers: EXPECTED_LANGUAGE_FRONTEND_WORKERS,
            route_handlers: EXPECTED_LANGUAGE_FRONTEND_ROUTES,
            ..LanguageFrontendSummary::default()
        },
        "language-frontend extract preflight drifted: {facts:?}"
    );
    assert_eq!(
        edges,
        LanguageFrontendSummary {
            files: EXPECTED_LANGUAGE_FRONTEND_FILES,
            edges: EXPECTED_LANGUAGE_FRONTEND_EDGES,
            queue_edges: EXPECTED_LANGUAGE_FRONTEND_QUEUE_EDGES,
            ..LanguageFrontendSummary::default()
        },
        "language-frontend edge preflight drifted: {edges:?}"
    );
    assert_eq!(
        globs,
        LanguageFrontendSummary {
            files: EXPECTED_LANGUAGE_FRONTEND_FILES,
            glob_matches: EXPECTED_LANGUAGE_FRONTEND_GLOB_MATCHES,
            ..LanguageFrontendSummary::default()
        },
        "language-frontend glob preflight drifted: {globs:?}"
    );

    let mut group = c.benchmark_group("language_frontends");
    group.throughput(Throughput::Elements(
        EXPECTED_LANGUAGE_FRONTEND_FILES as u64,
    ));
    group.bench_function("extract", |b| {
        b.iter(|| black_box(collect_language_frontend_facts(black_box(&fixture))));
    });
    group.bench_function("edges", |b| {
        b.iter(|| black_box(collect_language_frontend_edges(black_box(&fixture))));
    });
    group.bench_function("queue_glob_match", |b| {
        b.iter(|| black_box(match_language_frontend_queue_globs(black_box(&fixture))));
    });
    group.finish();
}
