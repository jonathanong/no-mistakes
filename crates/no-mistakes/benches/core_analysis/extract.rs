use super::shard;
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::codebase::dependencies::extract::extract_import_facts_from_program;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Dense callable source: functions, aliases, and class static members.
fn extract_fixture_source(callables: usize) -> String {
    let mut source = String::with_capacity(callables * 96);
    for index in 0..callables {
        source.push_str("export function f");
        source.push_str(&index.to_string());
        source.push_str("() { g");
        source.push_str(&index.to_string());
        source.push_str("(); C");
        source.push_str(&index.to_string());
        source.push_str(".m(); }\nfunction g");
        source.push_str(&index.to_string());
        source.push_str("() { f");
        source.push_str(&index.to_string());
        source.push_str("(); }\nconst a");
        source.push_str(&index.to_string());
        source.push_str(" = g");
        source.push_str(&index.to_string());
        source.push_str(";\nclass C");
        source.push_str(&index.to_string());
        source.push_str(" { static m() { a");
        source.push_str(&index.to_string());
        source.push_str("(); } }\n");
    }
    source
}

pub(super) fn bench_extract_import_facts(c: &mut Criterion) {
    if !shard::should_run(shard::GRAPH_CORE) {
        return;
    }
    let mut group = c.benchmark_group("extract/import_facts");
    // Memory-instrumented CI shards time out on a 2048-callable source.
    const CALLABLES: usize = 512;
    let source = extract_fixture_source(CALLABLES);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program(&parsed.program);
    assert!(
        facts.function_calls.len() >= CALLABLES * 4,
        "extract fixture must record calls, aliases, and class members"
    );
    group.throughput(Throughput::Elements(CALLABLES as u64));
    group.bench_with_input(
        BenchmarkId::from_parameter(CALLABLES),
        &parsed.program,
        |b, program| {
            b.iter(|| black_box(extract_import_facts_from_program(black_box(program))));
        },
    );
    group.finish();
}
