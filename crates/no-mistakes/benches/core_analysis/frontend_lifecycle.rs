//! Opt-in first-analysis accounting. The original benchmark is unchanged.
use criterion::{black_box, Criterion, IterManualOptions, Throughput};
use no_mistakes::benchmark_support::{
    collect_language_frontend_edges, collect_language_frontend_facts, language_frontend_fixture,
    match_language_frontend_queue_globs, LanguageFrontendSummary,
};

const MODE: &str = "NO_MISTAKES_FRONTEND_LIFECYCLE";

pub(super) fn selected() -> bool {
    std::env::var_os(MODE).is_some()
}

// This target runs first, before any shard can prepare fixtures or workers.
pub(super) fn validate_selection(_: &mut Criterion) {
    if !selected() {
        return;
    }
    assert_eq!(std::env::var(MODE).as_deref(), Ok("cold_first_analysis"));
    assert_eq!(
        std::env::var("NO_MISTAKES_BENCH_SHARD").as_deref(),
        Ok("language-frontends"),
        "lifecycle controls require the isolated language-frontends shard"
    );
    assert_eq!(std::env::var("RAYON_NUM_THREADS").as_deref(), Ok("4"));
    assert!(
        cfg!(codspeed) || std::env::args().any(|arg| arg == "--test"),
        "native lifecycle controls require --test: pinned Criterion cannot analyze one sample; use CodSpeed for tracked memory"
    );
}

pub(super) fn register_cold(c: &mut Criterion) {
    // Inventory/config preparation is outside the marker; no collector or Rayon
    // API runs before it. This is first-analysis cold, not OS process startup.
    let fixture = language_frontend_fixture();
    let mut calls = 0;
    let mut facts = LanguageFrontendSummary::default();
    let mut group = c.benchmark_group("language_frontend_lifecycle");
    group.throughput(Throughput::Elements(117));
    group.bench_function("cold_first_analysis", |b| {
        assert_eq!(calls, 0, "adapter reentered the cold closure");
        b.iter_manual_unstable(
            IterManualOptions::new().rounds(1).iters(1).warmup(0),
            || {
                assert_eq!(calls, 0, "unexpected adapter warmup");
                calls += 1;
                facts = black_box(collect_language_frontend_facts(black_box(&fixture)));
                facts
            },
        );
    });
    group.finish();
    assert_eq!(calls, 1, "cold benchmark was filtered out or repeated");
    assert_eq!(rayon::current_num_threads(), 4);
    assert_eq!(facts, expected_facts());
    // The original edge/glob preflight is retained after the cold stop so it
    // cannot silently condition the first analysis.
    let edges = collect_language_frontend_edges(&fixture);
    let globs = match_language_frontend_queue_globs(&fixture);
    assert_eq!(edges.files, 117);
    assert_eq!((edges.edges, edges.queue_edges), (125, 14));
    assert_eq!((globs.files, globs.glob_matches), (117, 117));
    println!(
        "FRONTEND_LIFECYCLE {{\"mode\":\"cold_first_analysis\",\"pid\":{},\"priorCollectors\":0,\"measuredClosures\":{},\"measuredCollectors\":1,\"postMarkerCollectors\":1,\"rayonWorkers\":4,\"files\":117,\"parsed\":69,\"imports\":66,\"enqueues\":7,\"workers\":6,\"routes\":41,\"edges\":125,\"queueEdges\":14,\"globMatches\":117}}",
        std::process::id(), calls
    );
}

fn expected_facts() -> LanguageFrontendSummary {
    LanguageFrontendSummary {
        files: 117,
        parsed_files: 69,
        imports: 66,
        enqueues: 7,
        workers: 6,
        route_handlers: 41,
        ..LanguageFrontendSummary::default()
    }
}
