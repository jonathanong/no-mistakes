use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

/// One live per-file `check_timeout` on the installing thread. The deadline is
/// not expired, so the call decodes one `Instant` and stays lock-free.
fn per_file_check_timeout(c: &mut Criterion) {
    let _guard = no_mistakes::invocation::install_test_deadline(Duration::from_secs(3_600))
        .expect("install live deadline");
    // One iteration is one call on every job. Wall time is the contended call,
    // not thread spawn: Criterion's iteration count amortizes the scope.
    let mut group = c.benchmark_group("per_file_check_timeout");
    for jobs in [1usize, 28] {
        group.bench_function(format!("{jobs}_jobs"), |b| {
            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                std::thread::scope(|scope| {
                    for _ in 0..jobs {
                        scope.spawn(|| {
                            for _ in 0..iters {
                                no_mistakes::invocation::check_timeout()
                                    .expect("deadline is still live");
                            }
                        });
                    }
                });
                start.elapsed()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, per_file_check_timeout);
criterion_main!(benches);
