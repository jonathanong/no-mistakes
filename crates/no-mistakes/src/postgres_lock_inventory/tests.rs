use super::*;
use std::path::Path;

fn fixture(name: &str) -> PathBuf {
    crate::test_support::fixture_root("postgres-lock-inventory-owner").join(name)
}

fn options(root: &Path, config: &str) -> PostgresLockInventoryOptions {
    PostgresLockInventoryOptions {
        root: root.to_path_buf(),
        config: Some(PathBuf::from(config)),
        write_baseline: false,
    }
}

#[test]
fn rejects_invalid_baseline_contents_and_paths() {
    let root = fixture("invalid");
    for (config, expected) in [
        ("syntax.yml", "expected ident"),
        ("version.yml", "unsupported version 2"),
        ("path.yml", "repository-relative path"),
        ("duplicate.yml", "duplicate baseline site"),
    ] {
        let error = run(&options(&root, config)).unwrap_err();
        assert!(
            format!("{error:#}").contains(expected),
            "{config}: {error:#}"
        );
    }
}

#[test]
fn normalizes_root_and_bootstraps_the_default_baseline_directory() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("queries"));
    let root = fixture.path();
    let mut write = options(&root.join("nested").join(".."), "config.yml");
    write.write_baseline = true;
    let report = run(&write).unwrap();
    assert!(report.baseline_updated);
    assert_eq!(report.baseline_file, ".no-mistakes/postgres-locks.json");
    assert!(root.join(&report.baseline_file).is_file());
    assert_eq!(report.sites.len(), 1);
    assert_eq!(report.sites[0].file, "queries.sql");
    let read = run(&options(root, "config.yml")).unwrap();
    assert_eq!(read.sites, report.sites);
    assert!(read.added.is_empty());
    assert!(read.removed.is_empty());
}

#[cfg(unix)]
#[test]
fn baseline_bootstrap_rejects_external_and_dangling_directory_symlinks() {
    let external = tempfile::tempdir().unwrap();
    for (target, expected) in [
        (external.path().to_path_buf(), "outside the repository"),
        (external.path().join("missing"), "broken symbolic link"),
    ] {
        let fixture = crate::test_support::materialize_saved_fixture(&fixture("queries"));
        let root = fixture.path();
        std::os::unix::fs::symlink(&target, root.join(".no-mistakes")).unwrap();
        let mut options = options(root, "config.yml");
        options.write_baseline = true;
        let error = run(&options).unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
        assert!(!target.join("postgres-locks.json").exists());
    }
}

#[test]
fn reports_sql_parse_and_read_gaps_alongside_other_files() {
    let base = fixture("queries");
    let root = base.parent().unwrap();
    let files = [
        base.join("queries.sql"),
        root.join("invalid/malformed.sql"),
        root.join("missing.sql"),
    ];
    let session = AnalysisSession::new(None);
    let config = session
        .config(root, Some(Path::new("queries/config.yml")))
        .unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        &files,
        std::sync::Arc::clone(&sources),
        &config,
        &[rule::RULE_ID],
    )
    .unwrap();
    let report =
        rule::inventory_with_files_sources_and_facts(root, &config, &files, &sources, &facts)
            .unwrap();
    assert_eq!(report.counts["for-update"], 1);
    assert_eq!(report.unanalyzed.len(), 2);
    assert_eq!(report.unanalyzed[0].file, "invalid/malformed.sql");
    assert!(report.unanalyzed[0].reason.contains("parsed"));
    assert_eq!(report.unanalyzed[1].file, "missing.sql");
    assert!(report.unanalyzed[1].reason.contains("read"));
}

#[test]
fn deleted_tracked_baseline_cannot_bootstrap_new_lock_sites() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("tracked"));
    let root = fixture.path();
    crate::test_support::git_init(root);
    crate::test_support::git_add_force(root, &["baseline.json", "config.yml", "queries.sql"]);
    std::fs::remove_file(root.join("baseline.json")).unwrap();
    let mut write = options(root, "config.yml");
    write.write_baseline = true;
    let error = run(&write).unwrap_err();
    assert!(format!("{error:#}").contains("tracked baseline cannot be reinitialized"));
    assert!(!root.join("baseline.json").exists());
}
