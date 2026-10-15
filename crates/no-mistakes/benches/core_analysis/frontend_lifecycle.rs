//! Opt-in lifecycle accounting. The original benchmark is unchanged.
use criterion::{black_box, Criterion, IterManualOptions, Throughput};
use no_mistakes::benchmark_support::{
    collect_language_frontend_edges, collect_language_frontend_facts, language_frontend_fixture,
    match_language_frontend_queue_globs, LanguageFrontendSummary,
};

#[path = "frontend_lifecycle/sequence.rs"]
mod sequence;
use sequence::{expected_facts, recorded_prefix, Prefix, Trace};

const MODE: &str = "NO_MISTAKES_FRONTEND_LIFECYCLE";
const COLD: &str = "cold_first_analysis";
const INCLUSIVE: &str = "conditioning_inclusive";
const INCREMENTAL: &str = "extract_after_recorded_prefix";

pub(super) fn selected() -> bool {
    std::env::var_os(MODE).is_some()
}

// This target runs first, before any shard can prepare fixtures or workers.
pub(super) fn validate_selection(_: &mut Criterion) {
    if !selected() {
        return;
    }
    assert!(matches!(
        std::env::var(MODE).as_deref(),
        Ok(COLD | INCLUSIVE | INCREMENTAL)
    ));
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

pub(super) fn register(c: &mut Criterion) {
    let mode = std::env::var(MODE).expect("validated lifecycle selection");
    let incremental = mode == INCREMENTAL;
    let inclusive = mode == INCLUSIVE;
    // Even fixture discovery uses Rayon. Cold/inclusive must initialize it
    // inside the marker; retain that SAME fixture through the marker stop.
    let mut fixture = None;
    let mut trace = Trace::default();
    let mut prefix = Prefix::default();
    if incremental {
        trace.push("fixture");
        fixture = Some(language_frontend_fixture());
        prefix = recorded_prefix(fixture.as_ref().expect("prepared fixture"), &mut trace);
    }
    let prior_events = trace.events().len();
    let mut calls = 0;
    let mut facts = LanguageFrontendSummary::default();
    let mut group = c.benchmark_group("language_frontend_lifecycle");
    group.throughput(Throughput::Elements(117));
    group.bench_function(&mode, |b| {
        assert_eq!(calls, 0, "adapter reentered the lifecycle closure");
        b.iter_manual_unstable(
            IterManualOptions::new().rounds(1).iters(1).warmup(0),
            || {
                assert_eq!(calls, 0, "unexpected adapter warmup");
                calls += 1;
                trace.push("manual-start");
                if !incremental {
                    trace.push("fixture");
                    fixture = Some(language_frontend_fixture());
                }
                let fixture = fixture.as_ref().expect("initialized fixture");
                if inclusive {
                    prefix = recorded_prefix(fixture, &mut trace);
                }
                trace.push("probe");
                facts = black_box(collect_language_frontend_facts(black_box(fixture)));
                facts
            },
        );
    });
    group.finish();
    assert_eq!(calls, 1, "lifecycle benchmark was filtered out or repeated");
    trace.push("manual-return");
    assert_eq!(rayon::current_num_threads(), 4);
    assert_eq!(facts, expected_facts());
    if inclusive || incremental {
        sequence::validate_prefix(prefix);
    } else {
        // Keep the original edge/glob preflight, after the cold marker stop.
        let fixture = fixture.as_ref().expect("retained cold fixture");
        trace.push("edges");
        let edges = collect_language_frontend_edges(fixture);
        trace.push("globs");
        let globs = match_language_frontend_queue_globs(fixture);
        sequence::validate_relationships(edges, globs);
    }
    // Helper order is observed. Nested collector counts are source-accounted:
    // each edge helper collects facts once; the glob helper does not.
    println!(
        "FRONTEND_LIFECYCLE {}",
        serde_json::json!({
            "mode": mode, "pid": std::process::id(), "measuredClosures": calls,
            "priorCollectors": if incremental { 7 } else { 0 },
            "measuredCollectors": if inclusive { 8 } else { 1 },
            "postMarkerCollectors": if inclusive || incremental { 0 } else { 1 },
            "collectorCountsSource": "inspected helper call paths",
            "priorHelperEvents": prior_events, "helperEvents": trace.events(),
            "fixtureInitializations": trace.events().iter().filter(|event| **event == "fixture").count(),
            "rayonWorkers": 4, "files": 117, "parsed": 69, "imports": 66,
            "enqueues": 7, "workers": 6, "routes": 41, "edges": 125,
            "queueEdges": 14, "globMatches": 117
        })
    );
}
