use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::codebase::ts_source::{
    discover_visible_paths, git_visible_files, ignore_walk_builder,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn benchmark_root() -> (PathBuf, Option<TempDir>) {
    if let Some(root) = std::env::var_os("NO_MISTAKES_DISCOVERY_BENCH_ROOT") {
        return (
            PathBuf::from(root)
                .canonicalize()
                .expect("discovery benchmark root should exist"),
            None,
        );
    }

    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/graph-gates")
        .canonicalize()
        .expect("graph-gates performance fixture should exist");
    let temp = TempDir::new().expect("create outside-Git benchmark directory");
    let root = temp.path().join("graph-gates");
    copy_fixture_tree(&source, &root);
    (root, Some(temp))
}

fn copy_fixture_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create copied benchmark directory");
    for entry in fs::read_dir(source).expect("read benchmark fixture") {
        let entry = entry.expect("read benchmark fixture entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type().expect("read fixture file type");
        if file_type.is_dir() {
            copy_fixture_tree(&source_path, &destination_path);
        } else if file_type.is_file() {
            fs::copy(source_path, destination_path).expect("copy benchmark fixture file");
        } else {
            panic!(
                "unexpected special file in benchmark fixture: {}",
                source_path.display()
            );
        }
    }
}

fn serial_fallback_paths(root: &Path) -> Vec<PathBuf> {
    let mut walker_root = root.to_path_buf();
    let mut paths = ignore_walk_builder(root)
        .hidden(false)
        .filter_entry(|entry| {
            no_mistakes::invocation::check_timeout().is_ok()
                && (entry.depth() == 0
                    || !entry
                        .file_type()
                        .is_some_and(|file_type| file_type.is_dir())
                    || entry.file_name() != ".git")
        })
        .build()
        .take_while(|_| no_mistakes::invocation::check_timeout().is_ok())
        .filter_map(Result::ok)
        .filter_map(|entry| {
            if entry.depth() == 0 {
                walker_root = entry.path().to_path_buf();
            }
            entry.file_type().and_then(|file_type| {
                (file_type.is_file() || file_type.is_symlink()).then(|| {
                    entry
                        .path()
                        .strip_prefix(&walker_root)
                        .map_or_else(|_| entry.path().to_path_buf(), |rel| root.join(rel))
                })
            })
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

pub(super) fn bench_fallback_discovery(c: &mut Criterion) {
    let (root, _temp) = benchmark_root();
    assert!(
        git_visible_files(&root).is_none(),
        "fallback discovery benchmark root must be outside Git: {}",
        root.display()
    );

    let expected = serial_fallback_paths(&root);
    assert!(
        !expected.is_empty(),
        "benchmark fixture should contain paths"
    );
    assert!(expected.windows(2).all(|pair| pair[0] < pair[1]));
    let source_paths = expected
        .iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    matches!(
                        extension,
                        "js" | "jsx" | "mjs" | "mts" | "cjs" | "cts" | "ts" | "tsx"
                    )
                })
        })
        .count();
    eprintln!(
        "fallback discovery preflight: root={}, paths={}, source_paths={}",
        root.display(),
        expected.len(),
        source_paths
    );

    let mut group = c.benchmark_group("fallback_discovery");
    group.throughput(Throughput::Elements(expected.len() as u64));
    for workers in [1, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .expect("construct benchmark Rayon pool");
        let actual = pool.install(|| discover_visible_paths(&root));
        assert_eq!(
            actual, expected,
            "parallel discovery with {workers} workers"
        );
        group.bench_function(format!("threads_{workers}"), |b| {
            b.iter(|| pool.install(|| black_box(discover_visible_paths(black_box(&root)))));
        });
    }
    group.finish();
}
