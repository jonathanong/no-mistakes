use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;

const CHANGED_FILES: &[&str] = &[
    "src/value.ts",
    "dotnet/src/App/Value.cs",
    "swift/App/Sources/App/Value.swift",
];

fn no_mistakes_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/impacted-checks/multi-framework")
}

fn run_impacted_checks(bin: &Path, root: &Path) -> usize {
    let output = Command::new(bin)
        .arg("impacted-checks")
        .args(CHANGED_FILES)
        .arg("--root")
        .arg(root)
        .arg("--format")
        .arg("json")
        .output()
        .expect("no-mistakes benchmark command should run");

    assert!(
        output.status.success(),
        "benchmark command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("benchmark stdout should be valid JSON");
    assert_eq!(report["checks"].as_array().map(Vec::len), Some(4));
    output.stdout.len()
}

pub fn bench_impacted_checks_multi_framework(c: &mut Criterion) {
    let bin = no_mistakes_bin();
    let root = fixture_root();
    c.bench_function("impacted_checks_multi_framework", |b| {
        b.iter(|| black_box(run_impacted_checks(black_box(&bin), black_box(&root))))
    });
}

criterion_group!(benches, bench_impacted_checks_multi_framework);
criterion_main!(benches);
