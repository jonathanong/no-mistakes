use super::fixtures::{fixture_root, source_files, EXPECTED_SOURCE_FILES};
use super::shard;
use criterion::{black_box, BatchSize, Criterion, Throughput};
use no_mistakes::codebase::check_facts::{
    collect_check_facts_with_graph_files_and_playwright, CheckFactPlan,
};
use no_mistakes::codebase::ts_source::facts::TsFactPlan;

/// Measure the public fact-collection boundary with one primary check scope
/// and a separate graph-only scope. The primary scope requests queue facts
/// using the fixture's configured `createQueue` factory; the graph-only scope
/// requests imports and symbols. Each iteration creates a fresh collector
/// session/source inventory, so request-local context and extraction costs are
/// included rather than hidden behind cached facts.
pub(super) fn bench_prepared_extraction_context(c: &mut Criterion) {
    if !shard::should_run(shard::GRAPH_CORE) {
        return;
    }

    let root = fixture_root();
    let all_files = source_files(&root);
    let primary_files = all_files
        .iter()
        .filter(|path| {
            path.ends_with("src/queues/create.ts")
                || path.ends_with("src/queues/email.ts")
                || path.ends_with("src/jobs/send.ts")
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(primary_files.len(), 3);
    let mut plan = CheckFactPlan {
        queue: true,
        queue_factory_names: vec!["createQueue".to_string()],
        graph: TsFactPlan::imports_and_symbols(),
        ..CheckFactPlan::default()
    };
    // The graph-only cohort has a different extraction plan and context from
    // the queue-aware primary cohort, as it does in prepared batch collection.
    let preflight = collect_check_facts_with_graph_files_and_playwright(
        &root,
        primary_files.clone(),
        all_files.clone(),
        plan.clone(),
        None,
    );
    assert_eq!(preflight.files().len(), primary_files.len());
    assert_eq!(preflight.stats.files_discovered, EXPECTED_SOURCE_FILES);
    assert_eq!(preflight.stats.files_parsed, EXPECTED_SOURCE_FILES);
    assert_eq!(preflight.stats.parse_errors, 0);

    let mut group = c.benchmark_group("check_facts_prepared_context");
    group.throughput(Throughput::Elements(EXPECTED_SOURCE_FILES as u64));
    group.bench_function("queue_primary_plus_graph_only_14_sources", |b| {
        b.iter_batched(
            || (primary_files.clone(), all_files.clone(), plan.clone()),
            |(files, graph_files, plan)| {
                // Every measured call creates a fresh disabled AnalysisSession
                // and SourceStore, so request-local context and extraction
                // work are included without timing benchmark-input cloning.
                black_box(collect_check_facts_with_graph_files_and_playwright(
                    black_box(&root),
                    black_box(files),
                    black_box(graph_files),
                    black_box(plan),
                    None,
                ))
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}
