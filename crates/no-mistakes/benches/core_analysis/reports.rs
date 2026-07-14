use super::fixtures::fixture_root;
use clap::Parser;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::codebase::symbols::{self, SymbolsArgs};
use no_mistakes::codebase::ts_source::discover_visible_paths;
use no_mistakes::codebase::workspaces;

pub(super) fn bench_symbols(c: &mut Criterion) {
    let root = fixture_root();
    let args = SymbolsArgs::try_parse_from([
        "symbols",
        "--root",
        root.to_str().expect("fixture root should be UTF-8"),
        "--tsconfig",
        root.join("tsconfig.json")
            .to_str()
            .expect("fixture tsconfig should be UTF-8"),
        "--include",
        "both",
        "src/app.tsx",
        "packages/core/src/index.ts",
    ])
    .expect("symbols benchmark arguments should parse");
    let expected = symbols::run_json(args).expect("symbols preflight should succeed");
    assert!(expected.contains("CoreValue"));

    c.bench_function("symbols/two_files", |b| {
        b.iter(|| {
            let args = SymbolsArgs::try_parse_from([
                "symbols",
                "--root",
                root.to_str().expect("fixture root should be UTF-8"),
                "--tsconfig",
                root.join("tsconfig.json")
                    .to_str()
                    .expect("fixture tsconfig should be UTF-8"),
                "--include",
                "both",
                "src/app.tsx",
                "packages/core/src/index.ts",
            ])
            .expect("symbols benchmark arguments should parse");
            black_box(symbols::run_json(args).expect("symbol extraction should succeed"))
        });
    });
}

pub(super) fn bench_workspace(c: &mut Criterion) {
    let root = fixture_root();
    let visible = discover_visible_paths(&root)
        .into_iter()
        .collect::<Vec<_>>();
    let preflight =
        workspaces::load_from_files(&root, &visible).expect("workspace preflight should succeed");
    assert_eq!(preflight.packages.len(), 1);
    assert_eq!(
        preflight.resolve_specifier("@fixture/core").as_deref(),
        Some(root.join("packages/core/src/index.ts").as_path())
    );

    let mut group = c.benchmark_group("workspace");
    group.throughput(Throughput::Elements(visible.len() as u64));
    group.bench_function("load_from_visible_files", |b| {
        b.iter(|| {
            black_box(
                workspaces::load_from_files(black_box(&root), black_box(&visible))
                    .expect("workspace load should succeed"),
            )
        });
    });
    group.bench_function("resolve_package", |b| {
        b.iter(|| black_box(preflight.resolve_specifier(black_box("@fixture/core"))));
    });
    group.finish();
}
