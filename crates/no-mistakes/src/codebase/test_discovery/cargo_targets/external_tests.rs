use super::*;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/cargo-external-targets"),
    )
}

#[test]
fn cargo_external_entries_union_declaring_and_physical_owners_with_authoritative_scopes() {
    let root = fixture();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    for (scopes, shared, foreign) in [
        (
            vec![],
            vec!["declaring-a", "declaring-b"],
            vec!["declaring-a", "declaring-b"],
        ),
        (vec!["a"], vec!["declaring-a"], vec!["declaring-a"]),
        (vec!["b"], vec!["declaring-b"], vec!["declaring-b"]),
        (vec!["absent"], vec![], vec![]),
    ] {
        let mut config = NoMistakesConfig::default();
        config.tests.rust.packages = scopes.iter().map(|scope| (*scope).into()).collect();
        let catalog = CargoTargets::prepare(&root, &config, &visible, &sources);
        let discovered = catalog.discover(&root, &visible);
        for (file, expected) in [
            ("shared/tests/smoke.rs", shared),
            ("b/tests/physical.rs", foreign),
        ] {
            let targets = discovered
                .targets_by_path
                .get(&root.join(file))
                .cloned()
                .unwrap_or_default();
            assert_eq!(
                targets
                    .iter()
                    .filter_map(|target| target.project.as_deref())
                    .collect::<Vec<_>>(),
                expected,
                "{scopes:?}: {file}"
            );
            assert!(targets
                .iter()
                .all(|target| target.runner_args[0] == "--manifest-path"));
        }
        if scopes.is_empty() || scopes == ["a"] {
            assert_eq!(
                discovered.targets_by_path[&root.join("shared/entry.rs")][0].runner_args,
                [
                    "--manifest-path",
                    "a/Cargo.toml",
                    "-p",
                    "declaring-a",
                    "--test",
                    "unusual"
                ]
            );
            for file in ["shared/tests/disabled.rs", "shared/tests/feature.rs"] {
                assert!(discovered.targets_by_path[&root.join(file)].is_empty());
            }
        } else {
            assert!(!discovered.tests.contains(&root.join("shared/entry.rs")));
        }
        assert!(!discovered.tests.contains(&root.join("shared/helper.rs")));
        assert!(!discovered.tests.contains(&root.join("shared/missing.rs")));
        let reads = sources.physical_read_count();
        for _ in 0..16 {
            catalog.discover(&root, &visible);
        }
        assert_eq!(sources.physical_read_count(), reads);
        assert_eq!(
            catalog.external_entries.len(),
            match scopes.as_slice() {
                [] | ["a"] => 5,
                ["b"] => 1,
                _ => 0,
            }
        );
        assert!(!discovered.used_fallback);
    }
}
