// Reproduction asset for swift-corpus-measurement.md.
use criterion::{black_box, Criterion};
use no_mistakes::benchmark_support::{
    collect_language_frontend_facts, collect_swift_frontend_facts, language_frontend_fixture,
    native_frontend_fixture,
};

fn main() {
    let native = native_frontend_fixture();
    let language = language_frontend_fixture();
    let swift = collect_swift_frontend_facts(&native);
    assert_eq!(
        (swift.files, swift.parsed_files, swift.physical_reads),
        (15, 5, 7)
    );
    assert_eq!(collect_language_frontend_facts(&language).parsed_files, 69);
    let mut criterion = Criterion::default()
        .sample_size(30)
        .warm_up_time(std::time::Duration::from_secs(2))
        .measurement_time(std::time::Duration::from_secs(4))
        .configure_from_args();
    criterion.bench_function("swift_corpus", |bench| {
        bench.iter(|| black_box(collect_swift_frontend_facts(black_box(&native))))
    });
    criterion.bench_function("language_control", |bench| {
        bench.iter(|| black_box(collect_language_frontend_facts(black_box(&language))))
    });
    criterion.final_summary();
}
