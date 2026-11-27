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
    for (scenario, config, expected) in [
        ("invalid", "syntax.yml", "expected ident"),
        ("invalid", "version.yml", "unsupported version 2"),
        ("invalid", "path.yml", "repository-relative path"),
        ("invalid", "absolute.yml", "repository-relative path"),
        ("invalid", "duplicate.yml", "duplicate baseline site"),
        (
            "invalid",
            "empty-identity.yml",
            "empty baseline site identity",
        ),
        (
            "invalid",
            "empty-helper.yml",
            "helpers must be non-empty names",
        ),
        (
            "invalid",
            "unanalyzed-zero.yml",
            "baseline unanalyzed count must be positive",
        ),
        (
            "invalid",
            "unanalyzed-duplicate.yml",
            "duplicate baseline unanalyzed entry",
        ),
        (
            "invalid",
            "unanalyzed-empty.yml",
            "empty baseline unanalyzed identity",
        ),
        (
            "invalid",
            "invalid-baseline-unanalyzed.yml",
            "baselineUnanalyzed",
        ),
        (
            "queries",
            "directory-baseline.yml",
            "must be a regular file",
        ),
    ] {
        let error = run(&options(&fixture(scenario), config)).unwrap_err();
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
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "save the initial SQL fixture");
    let mut write = options(&root.join("nested").join(".."), "config.yml");
    write.write_baseline = true;
    let report = run(&write).unwrap();
    assert!(report.baseline_updated);
    assert_eq!(report.baseline_file, ".no-mistakes/postgres-locks.json");
    assert_eq!(report.sites.len(), 1);
    let site = &report.sites[0];
    assert_eq!(site.file, "queries.sql");
    let bytes = std::fs::read(root.join(&report.baseline_file)).unwrap();
    let written: LockInventoryBaseline = serde_json::from_slice(&bytes).unwrap();
    let mut expected = LockInventoryBaseline::default();
    assert_eq!(expected.version, 1);
    expected.sites.push(LockInventoryBaselineSite {
        key: site.key.clone(),
        file: site.file.clone(),
        category: site.category.clone(),
    });
    assert_eq!(written, expected);
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
    assert_eq!(report.unanalyzed_by_directory["invalid"], 1);
    assert_eq!(report.unanalyzed_by_directory["."], 1);
    assert_eq!(
        report.unanalyzed_by_reason["SQL could not be parsed completely"],
        1
    );
    assert_eq!(
        report.unanalyzed_by_reason["SQL source could not be read"],
        1
    );
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

#[test]
fn staged_baseline_deletion_cannot_bootstrap_new_lock_sites() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("tracked"));
    let root = fixture.path();
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "save an empty lock baseline");
    git(root, &["rm", "--", "baseline.json"]);

    let mut write = options(root, "config.yml");
    write.write_baseline = true;
    let error = run(&write).unwrap_err();
    assert!(format!("{error:#}").contains("tracked baseline cannot be reinitialized"));
    assert!(!root.join("baseline.json").exists());

    git(
        root,
        &[
            "restore",
            "--source=HEAD",
            "--staged",
            "--worktree",
            "--",
            "baseline.json",
        ],
    );
    let error = run(&write).unwrap_err();
    assert!(format!("{error:#}").contains("baseline may only go down"));
    assert!(root.join("baseline.json").is_file());
}

#[test]
fn inventories_only_the_configured_transaction_callback() {
    let report = run(&options(&fixture("callbacks"), "config.yml")).unwrap();
    let [site] = report.added.as_slice() else {
        panic!("expected one callback lock, got {:?}", report.added);
    };
    assert_eq!(site.file, "queries.ts");
    assert_eq!(site.line, 5);
    assert_eq!(site.category, "for-update");
    assert_eq!(
        report.unanalyzed,
        [UnanalyzedLockSite {
            file: "queries.ts".into(),
            line: 12,
            reason: "executor SQL could not be resolved".into(),
        }]
    );
}

fn git(root: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
