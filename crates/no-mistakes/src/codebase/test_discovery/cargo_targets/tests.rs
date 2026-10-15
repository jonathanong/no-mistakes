use super::*;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/cargo-target-ownership"),
    )
}
#[test]
fn cargo_manifests_define_package_and_real_target_identity() {
    let root = fixture();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let catalog = CargoTargets::prepare(&root, &NoMistakesConfig::default(), &visible, &sources);
    assert_eq!(
        sources.physical_read_count(),
        8,
        "one read per visible manifest"
    );
    assert!(
        catalog.projects(&root).is_empty(),
        "source fallback requires configured packages"
    );
    assert_eq!(catalog.membership_index_builds, 1);
    let discovered = catalog.discover(&root, &visible);
    for (file, package, flags) in [
        (
            "editions/implicit-workspace/tests/automatic.rs",
            "implicit-workspace",
            vec!["--test", "automatic"],
        ),
        (
            "editions/inherited/src/bin/inherited-package/tests.rs",
            "inherited-package",
            vec!["--bin", "inherited-package"],
        ),
        (
            "editions/inherited/tests/automatic.rs",
            "inherited-package",
            vec!["--test", "automatic"],
        ),
        ("editions/e2015/src/tests.rs", "edition-2015", vec!["--lib"]),
        (
            "editions/e2015/src/bin/helper/tests.rs",
            "edition-2015",
            vec!["--bin", "helper"],
        ),
        (
            "editions/e2024/tests/physical.rs",
            "edition-2024",
            vec!["--test", "renamed-entry"],
        ),
        (
            "editions/e2015/tests/manual.rs",
            "edition-2015",
            vec!["--test", "manual"],
        ),
        (
            "editions/e2024/tests/automatic.rs",
            "edition-2024",
            vec!["--test", "automatic"],
        ),
        ("editions/e2024/src/tests.rs", "edition-2024", vec!["--lib"]),
        (
            "crates/engine/src/bin/worker/tests.rs",
            "real-engine",
            vec!["--bin", "worker"],
        ),
        (
            "crates/engine/tests/smoke.rs",
            "real-engine",
            vec!["--test", "smoke"],
        ),
        (
            "crates/custom/harness/suite/entry.rs",
            "renamed-package",
            vec!["--test", "declared-suite"],
        ),
        (
            "crates/custom/harness/suite/tests.rs",
            "renamed-package",
            vec!["--test", "declared-suite"],
        ),
        (
            "crates/custom/library/tests.rs",
            "renamed-package",
            vec!["--lib"],
        ),
        (
            "tools/console/src/tests.rs",
            "real-console",
            vec!["--bin", "real-console"],
        ),
    ] {
        let targets = &discovered.targets_by_path[&root.join(file)];
        assert!(
            targets
                .iter()
                .any(|target| target.project.as_deref() == Some(package)
                    && target.runner_args.ends_with(
                        &flags
                            .iter()
                            .map(|flag| flag.to_string())
                            .collect::<Vec<_>>()
                    )),
            "{file}: {targets:?}"
        );
        assert_eq!(targets.len(), 1, "{file}: no duplicate inferred target");
        assert!(targets
            .iter()
            .all(|target| target.runner_args.contains(&"--manifest-path".to_string())));
    }
    for file in [
        "crates/engine/src/tests.rs",
        "crates/engine/src/unit/tests/mod.rs",
        "crates/engine/tests/helpers/common.rs",
        "crates/custom/harness/disabled.rs",
        "crates/custom/harness/feature.rs",
        "crates/custom/tests/disabled.rs",
        "crates/custom/src/bin/ignored/tests.rs",
        "orphan/tests/example.rs",
        "editions/e2015/tests/automatic.rs",
    ] {
        assert!(
            discovered.targets_by_path[&root.join(file)].is_empty(),
            "{file}"
        );
    }
    assert!(
        discovered.targets_by_path[&root.join("crates/engine/tests/helpers/common.rs")]
            .iter()
            .all(|target| !target.runner_args.contains(&"common".to_string()))
    );
    for _ in 0..16 {
        catalog.discover(&root, &visible);
    }
    assert_eq!(
        sources.physical_read_count(),
        8,
        "queries reuse the prepared manifest projection"
    );
}
#[test]
fn cargo_explicit_package_scope_stays_authoritative() {
    let root = fixture();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let mut config = NoMistakesConfig::default();
    config.tests.rust.packages = vec!["crates/engine".into()];
    let catalog = CargoTargets::prepare(&root, &config, &visible, &sources);
    let discovered = catalog.discover(&root, &visible);
    assert!(discovered
        .tests
        .iter()
        .all(|file| file.starts_with(root.join("crates/engine"))));
    assert_eq!(sources.physical_read_count(), 1);
    let projects = catalog.projects(&root);
    assert_eq!(projects.len(), 1);
    assert_eq!(
        projects[0].runner_project_arg.as_deref(),
        Some("real-engine")
    );
}

#[test]
fn cargo_target_manifest_boundaries_and_default_paths() {
    let root = fixture().with_file_name("cargo-target-edge-cases");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let mut config = NoMistakesConfig::default();
    config.tests.rust.packages = vec![".".into()];
    let catalog = CargoTargets::prepare(&root, &config, &visible, &sources);
    assert_eq!(catalog.projects(&root)[0].config.as_deref(), Some("."));
    // Invalid nested manifests remain ownership barriers, never borrowing a parent package.
    let unscoped = CargoTargets::prepare(&root, &NoMistakesConfig::default(), &visible, &sources);
    assert!(
        unscoped
            .targets(
                &root,
                &root,
                &unscoped.packages[&root],
                &root.join("src/tests.rs")
            )
            .is_empty(),
        "shared source does not establish a target"
    );
    let package = &unscoped.packages[&root];
    for (path, name) in [
        ("src/main.rs", "edge-package"),
        ("src/bin/explicit.rs", "explicit"),
        ("src/bin/folder/tests.rs", "folder"),
    ] {
        let targets = unscoped.targets(&root, &root, package, &root.join(path));
        assert_eq!(targets.len(), 1, "{path}");
        assert!(targets[0]
            .runner_args
            .ends_with(&["--bin".into(), name.into()]));
    }
    let discovered = unscoped.discover(&root, &visible);
    for path in [
        "malformed/tests/example.rs",
        "unnamed/tests/example.rs",
        "invalid-target/tests/example.rs",
    ] {
        assert!(discovered.targets_by_path[&root.join(path)].is_empty());
    }
    assert_eq!(
        sources.physical_read_count(),
        4,
        "failed manifests keep one source identity"
    );
}

#[test]
fn cargo_missing_manifest_read_keeps_one_failure_identity() {
    let root = fixture().with_file_name("cargo-target-edge-cases");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let mut visible = snapshot.paths_for(&root).to_vec();
    // The inventory can outlive a deleted Cargo.toml. Do not fall back to its parent package.
    let missing = root.join("missing/Cargo.toml");
    visible.push(missing.clone());
    let sources = snapshot.source_store_for(&root);
    let catalog = CargoTargets::prepare(&root, &NoMistakesConfig::default(), &visible, &sources);
    assert!(catalog.manifests.contains(missing.parent().unwrap()));
    assert!(!catalog.packages.contains_key(missing.parent().unwrap()));
    let reads = sources.physical_read_count();
    assert!(sources.read_path(&missing).is_err());
    assert_eq!(sources.physical_read_count(), reads);
}

#[test]
fn cargo_workspace_edition_does_not_read_outside_prepared_visibility() {
    let root = fixture();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root);
    let workspace = root.join("Cargo.toml");
    let visible = files
        .iter()
        .map(PathBuf::as_path)
        .filter(|path| *path != workspace)
        .collect();
    let sources = snapshot.source_store_for(&root);
    let mut manifests = manifests::Manifests::new(&sources);
    let package = root.join("editions/inherited/Cargo.toml");
    assert_eq!(manifests.edition(&package, &visible), None);
    assert_eq!(
        sources.physical_read_count(),
        1,
        "the hidden workspace must stay unread"
    );
}

#[test]
fn cargo_invalid_workspace_edition_keeps_unknown_identity() {
    let root = fixture().with_file_name("cargo-edition-boundaries");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root);
    let visible = files.iter().map(PathBuf::as_path).collect();
    let sources = snapshot.source_store_for(&root);
    let mut manifests = manifests::Manifests::new(&sources);
    for _ in 0..16 {
        for path in [
            "false/Cargo.toml",
            "missing-package/member/Cargo.toml",
            "missing-edition/member/Cargo.toml",
        ] {
            assert_eq!(manifests.edition(&root.join(path), &visible), None);
        }
    }
    assert_eq!(sources.physical_read_count(), 5);
}

#[test]
fn cargo_absent_manifest_keeps_preparation_cold_and_scope_explicit() {
    let root = fixture();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files: Vec<_> = snapshot
        .paths_for(&root)
        .iter()
        .filter(|path| path.file_name().is_none_or(|name| name != "Cargo.toml"))
        .cloned()
        .collect();
    let sources = snapshot.source_store_for(&root);
    let mut config = NoMistakesConfig::default();
    let catalog = CargoTargets::prepare(&root, &config, &files, &sources);
    assert_eq!(catalog.membership_index_builds, 0);
    let unknown = catalog.discover(&root, &files);
    assert!(!unknown.tests.is_empty());
    assert!(unknown.targets_by_path.values().all(Vec::is_empty));
    config.tests.rust.packages = vec!["crates/engine".into()];
    let scoped = CargoTargets::prepare(&root, &config, &files, &sources);
    assert_eq!(scoped.membership_index_builds, 0);
    assert!(scoped.scopes.contains(&root.join("crates/engine")));
    assert!(scoped.discover(&root, &files).tests.is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}
