use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig")
        .join(name)
}

#[test]
fn canonical_visible_paths_use_multiple_workers_without_changing_membership() {
    let root = fixture("symlink-workspace");
    let paths = [
        root.join("link/tsconfig.json"),
        root.join("real/tsconfig.json"),
        root.join("link/src/entry.ts"),
        root.join("missing.ts"),
    ]
    .into_iter()
    .cycle()
    .take(128)
    .collect::<Vec<_>>();
    let expected = paths
        .iter()
        .filter_map(|path| real_path(path))
        .collect::<BTreeSet<_>>();
    let workers = AtomicUsize::new(0);
    let calls = AtomicUsize::new(0);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let actual = pool.install(|| {
        visible_real_paths(&paths, |path| {
            calls.fetch_add(1, Ordering::Relaxed);
            workers.fetch_or(
                1 << rayon::current_thread_index().unwrap(),
                Ordering::Relaxed,
            );
            // Give other workers time to take independent filesystem operations.
            // The serial implementation returns equal values but uses only one worker.
            std::thread::sleep(std::time::Duration::from_millis(1));
            real_path(path)
        })
    });
    assert_eq!(actual, expected);
    assert_eq!(calls.load(Ordering::Relaxed), paths.len());
    assert!(workers.load(Ordering::Relaxed).count_ones() > 1);
}

#[test]
fn parallel_catalog_preserves_symlink_extends_and_visible_universe() {
    let root = normalize_path(&fixture("symlink-workspace/link"));
    let paths = [
        "tsconfig.json",
        "tsconfig.base.json",
        "src/entry.ts",
        "src/value.ts",
        "project/tsconfig.json",
        "project/src/entry.ts",
        "project/src/value.ts",
        "missing.ts",
    ]
    .map(|path| root.join(path));
    let candidate_roots = [root.clone(), root.join("project")];
    let expected_visible = paths
        .iter()
        .map(|path| normalize_path(path))
        .collect::<BTreeSet<_>>();
    let expected_real = paths
        .iter()
        .filter_map(|path| real_path(path))
        .collect::<BTreeSet<_>>();
    let mut snapshots = Vec::new();
    for threads in [1, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        snapshots.push(pool.install(|| {
            let builder = CatalogBuilder::new(&root, &candidate_roots, &paths, None, None);
            assert_eq!(builder.visible, expected_visible);
            assert_eq!(builder.visible_real, expected_real);
            let catalog = builder.build();
            (
                paths
                    .iter()
                    .map(|path| catalog.provenance_for(path))
                    .collect::<Vec<_>>(),
                catalog
                    .configs
                    .iter()
                    .map(|config| {
                        (
                            config.path.clone(),
                            config.identity.clone(),
                            config.config.dir.clone(),
                            config.config.paths.clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
                catalog.diagnostics(),
            )
        }));
    }
    assert_eq!(snapshots[0], snapshots[1]);
    assert_eq!(snapshots[0].0[2].config, Some(root.join("tsconfig.json")));
    assert_eq!(
        snapshots[0].0[5].config,
        Some(root.join("project/tsconfig.json"))
    );
    assert!(snapshots[0]
        .1
        .iter()
        .any(|config| config.1.contains(&root.join("tsconfig.base.json"))));
}
