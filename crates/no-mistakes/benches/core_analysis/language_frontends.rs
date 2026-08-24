use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{
    collect_dotnet_frontend_facts, collect_language_frontend_edges,
    collect_language_frontend_facts, collect_swift_frontend_facts, language_frontend_fixture,
    match_language_frontend_queue_globs, native_frontend_fixture, LanguageFrontendSummary,
    NativeFrontendSummary,
};

pub(super) const EXPECTED_LANGUAGE_FRONTEND_FILES: usize = 117;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_PARSED: usize = 69;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_IMPORTS: usize = 66;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_ENQUEUES: usize = 7;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_WORKERS: usize = 6;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_ROUTES: usize = 41;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_EDGES: usize = 125;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_QUEUE_EDGES: usize = 14;
pub(super) const EXPECTED_LANGUAGE_FRONTEND_GLOB_MATCHES: usize = 117;

pub(super) fn bench_language_frontends(c: &mut Criterion) {
    // Keep this wrapper stable: criterion_group includes its name in CodSpeed
    // identities. The shard gates only isolate setup and measurement work.
    if shard::should_run(shard::LANGUAGE_FRONTENDS) {
        register_language_frontends(c);
    }

    if shard::should_run(shard::NATIVE_FRONTENDS) {
        register_native_frontends(c);
    }
}

fn register_language_frontends(c: &mut Criterion) {
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

fn register_native_frontends(c: &mut Criterion) {
    let native_fixture = native_frontend_fixture();
    let swift = collect_swift_frontend_facts(&native_fixture);
    let dotnet = collect_dotnet_frontend_facts(&native_fixture);
    assert_eq!(
        swift,
        NativeFrontendSummary {
            files: 15,
            parsed_files: 5,
            physical_reads: 7,
        },
        "update the stable Swift benchmark preflight after intentional fixture changes"
    );
    assert_eq!(
        dotnet,
        NativeFrontendSummary {
            files: 12,
            parsed_files: 5,
            physical_reads: 7,
        },
        "update the stable .NET benchmark preflight after intentional fixture changes"
    );

    let mut native_group = c.benchmark_group("native_frontends");
    native_group.bench_function("swift_facts", |b| {
        b.iter(|| black_box(collect_swift_frontend_facts(black_box(&native_fixture))));
    });
    native_group.bench_function("dotnet_facts", |b| {
        b.iter(|| black_box(collect_dotnet_frontend_facts(black_box(&native_fixture))));
    });
    native_group.finish();
}
