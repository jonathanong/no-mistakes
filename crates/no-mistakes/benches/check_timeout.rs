use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

/// One live per-file `check_timeout` on the installing thread. The deadline is
/// not expired, so the call decodes one `Instant` and stays lock-free.
fn per_file_check_timeout(c: &mut Criterion) {
    let _guard = no_mistakes::invocation::install_test_deadline(Duration::from_secs(3_600))
        .expect("install live deadline");
    c.bench_function("per_file_check_timeout", |b| {
        b.iter(|| {
            no_mistakes::invocation::check_timeout().expect("deadline is still live");
        });
    });
}

criterion_group!(benches, per_file_check_timeout);
criterion_main!(benches);
