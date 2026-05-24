use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;

fn no_mistakes_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn run_check(bin: &Path, root: &Path) -> usize {
    let output = Command::new(bin)
        .arg("check")
        .arg("--root")
        .arg(root)
        .arg("--format")
        .arg("json")
        .arg("--timings")
        .output()
        .expect("no-mistakes benchmark command should run");

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "benchmark command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.stderr.is_empty(),
        "--timings should emit phase timing output"
    );
    output.stdout.len()
}

pub fn bench_check_no_mistakes_repo(c: &mut Criterion) {
    let bin = no_mistakes_bin();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    c.bench_function("check_no_mistakes_repo", |b| {
        b.iter(|| black_box(run_check(black_box(&bin), black_box(&root))))
    });
}

criterion_group!(benches, bench_check_no_mistakes_repo);
criterion_main!(benches);
