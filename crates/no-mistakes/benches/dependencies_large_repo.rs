use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;

const ROOTS: &[&str] = &[
    "apps/web/src/entrypoints/graph-smoke.tsx",
    "apps/api/src/entrypoints/public-api.mts",
    "scripts/orchestrate.mts",
    "tests/e2e/all-routes.spec.ts",
    ".github/workflows/ci.yml",
    "README.md",
];

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis/large-graph-monorepo")
}

fn no_mistakes_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn run_dependencies_large_repo(bin: &Path, root: &Path) -> usize {
    let output = Command::new(bin)
        .arg("-j")
        .arg("4")
        .arg("dependencies")
        .arg("--root")
        .arg(root)
        .arg("--relationship")
        .arg("all")
        .arg("--timings")
        .arg("--format")
        .arg("json")
        .args(ROOTS)
        .output()
        .expect("no-mistakes benchmark command should run");

    assert!(
        output.status.success(),
        "benchmark command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.stderr.is_empty(),
        "--timings should emit phase timing output"
    );
    output.stdout.len()
}

pub fn bench_dependencies_large_repo(c: &mut Criterion) {
    let bin = no_mistakes_bin();
    let root = fixture_root();
    c.bench_function("dependencies_relationship_all_large_repo", |b| {
        b.iter(|| {
            black_box(run_dependencies_large_repo(
                black_box(&bin),
                black_box(&root),
            ))
        })
    });
}

criterion_group!(benches, bench_dependencies_large_repo);
criterion_main!(benches);
