use super::*;

#[test]
fn all_overlapping_entries_count_as_used() {
    let usage = AllowUsage::new(3);
    assert!(usage.suppress([true, true, false].into_iter()));
    assert!(!usage.suppress([false, false, false].into_iter()));
    assert_eq!(usage.unused().collect::<Vec<_>>(), [2]);
}

#[test]
fn coverage_requires_every_matching_inventory_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = [root.join("Cargo.toml"), root.join("src/lib.rs")];
    let sources = super::super::source_store_for_files(&files);
    let partial = CheckedScope::new(root, Some(&sources), &files[..1]);
    assert!(!partial.glob("**/*"));
    assert!(partial.exact("Cargo.toml"));
    assert!(!partial.exact("missing.rs"));
    assert!(!partial.glob("missing/**"));
    assert!(!partial.glob("["));
    let full = CheckedScope::new(root, Some(&sources), &files);
    assert!(full.glob("**/*"));
    assert!(!CheckedScope::new(root, None, &files).glob("**/*"));
}

#[test]
fn lexical_roots_share_inventory_and_checked_identities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/..");
    let files = [root.join("Cargo.toml")];
    let sources = super::super::source_store_for_files(&files);
    let scope = CheckedScope::new(&root, Some(&sources), &files);
    assert!(scope.exact("Cargo.toml"));
    assert!(scope.glob("*.toml"));
}

#[test]
fn an_empty_exception_list_needs_no_sources_or_scope_work() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = super::super::source_store_for_files(&[root.join("Cargo.toml")]);
    let config = NoMistakesConfig::default();
    let rule = RuleDef::default();
    let audit = ExceptionAudit::new(root, &config, &rule, &sources, "allow", &[]);
    assert!(audit.finish(&sources, |_| true).is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn missing_literals_require_absence_from_inventory_and_disk_in_the_selected_scope() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check/unused-allow-missing/fixture"),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule = &config.rules[0];
    let files = [
        root.join("src/live.mts"),
        root.join("src/inventory-only.mts"),
    ];
    let sources = super::super::source_access::complete_source_store_for_files(&files);
    let scope = MissingFiles::new(&root, &config, rule, Some(&sources)).unwrap();
    assert!(scope.contains("src/deleted.mts"));
    assert!(!scope.contains("src/inventory-only.mts"));
    assert!(!scope.contains("src/excluded.mts"));
    assert!(!scope.contains("src/live.mts"));
    assert!(!scope.contains("src/on-disk.mts"));
    assert!(!scope.contains("src"));
    assert!(!scope.contains("src/unmatched-*.mts"));
    assert!(!scope.contains("../outside.mts"));
    assert!(!scope.contains(""));
    assert!(!scope.contains("other/deleted.mts"));
    assert!(MissingFiles::new(&root, &config, rule, None).is_none());
    let partial = super::super::source_store_for_files(&files);
    assert!(MissingFiles::new(&root, &config, rule, Some(&partial)).is_none());
    let mut ignored = config.clone();
    ignored.report_unused_allow_entries = crate::config::v2::DisableDirectivePolicy::Ignore;
    assert!(MissingFiles::new(&root, &ignored, rule, Some(&sources)).is_none());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn missing_literals_respect_skipped_directories_and_invalid_filters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = super::super::source_access::complete_source_store_for_files(&[]);
    let mut config = NoMistakesConfig::default();
    let mut rule = RuleDef {
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    };
    let scope = MissingFiles::new(root, &config, &rule, Some(&sources)).unwrap();
    assert!(!scope.contains("node_modules/deleted.ts"));
    config.filesystem.skip_directories.push("retired".into());
    let scope = MissingFiles::new(root, &config, &rule, Some(&sources)).unwrap();
    assert!(!scope.contains("retired/deleted.ts"));
    rule.include = vec!["[".into()];
    assert!(MissingFiles::new(root, &config, &rule, Some(&sources)).is_none());
}

#[test]
fn missing_scope_reuses_skip_filters_and_explicit_project_roots() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = super::super::source_access::complete_source_store_for_files(&[]);
    let mut config = NoMistakesConfig::default();
    config.filesystem.skip_directories.push("generated".into());
    let mut rule = RuleDef {
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        exclude: vec!["**/*.test.*".into()],
        ..Default::default()
    };
    let scope = MissingFiles::new(root, &config, &rule, Some(&sources)).unwrap();
    assert!(scope.includes_path(&root.join("src/live.mts")));
    for file in [
        "fixtures/helper.mts",
        "generated/helper.mts",
        "src/ignored.test.mts",
    ] {
        assert!(!scope.includes_path(&root.join(file)), "{file}");
    }
    config.projects.insert(
        "saved".into(),
        crate::config::v2::schema::Project {
            root: Some("fixtures".into()),
            ..Default::default()
        },
    );
    rule.scope = None;
    rule.projects = vec!["saved".into()];
    let scope = MissingFiles::new(root, &config, &rule, Some(&sources)).unwrap();
    assert!(scope.includes_path(&root.join("fixtures/helper.mts")));
    assert!(!scope.includes_path(&root.join("src/live.mts")));
}
