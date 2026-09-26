use super::super::selection::{scan_with_sources, select_shell_candidates};
use super::*;
use crate::codebase::rules::{filesystem_dispatch, path_filter};

fn tracked_fixture() -> tempfile::TempDir {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shellcheck-runner/tracked-only");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    std::fs::rename(
        fixture.path().join(".gitignore.fixture"),
        fixture.path().join(".gitignore"),
    )
    .unwrap();
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_force(
        fixture.path(),
        &[
            ".gitignore",
            "scripts/nested/tracked.sh",
            "scripts/bash",
            "scripts/posix",
            "scripts/modified.sh",
            "scripts/staged.sh",
            "scripts/unsupported",
            "scripts/explicit",
        ],
    );
    std::fs::copy(
        fixture.path().join("modified-after-stage.fixture"),
        fixture.path().join("scripts/modified.sh"),
    )
    .unwrap();
    fixture
}

fn candidate_names(paths: &[PathBuf], root: &Path) -> Vec<String> {
    let mut names = paths
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn tracked_options() -> Options {
    Options {
        tracked_only: true,
        shebang_dirs: vec!["scripts".to_string()],
        shell_files: vec![
            "scripts/explicit".to_string(),
            "scripts/untracked-explicit".to_string(),
            "scripts/ignored-explicit".to_string(),
        ],
        ..Default::default()
    }
}

#[test]
fn unconfigured_rule_has_no_findings() {
    let fixture = tracked_fixture();
    assert!(check(
        fixture.path(),
        &crate::config::v2::NoMistakesConfig::default()
    )
    .unwrap()
    .is_empty());
}

#[test]
fn tracked_only_rejects_non_boolean_options_in_both_paths() {
    let root = fixture_root("pass");
    let config = config_with_rule("{trackedOnly: not-a-bool}");
    let files = Vec::new();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = crate::codebase::rules::source_store_for_files(&files);

    assert!(check(&root, &config).is_err());
    assert!(
        check_with_files_sources_and_snapshot(&root, &config, &files, &sources, &snapshot).is_err()
    );
}

#[test]
fn prepared_scan_with_no_candidates_has_no_findings() {
    let root = fixture_root("pass");
    let config = config_with_rule("{}");
    let rule_filter = path_filter::RulePathFilter::new(&root, &config, &config.rules[0]).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&[]);

    assert!(scan_with_sources(
        &root,
        &Options::default(),
        &[],
        &[],
        &rule_filter,
        &sources,
        &snapshot,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn multiple_shellcheck_findings_are_sorted() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let nested = root.join("scripts/nested/tracked.sh");
    let staged = root.join("scripts/staged.sh");
    let output = std::process::Output {
        status: make_exit_status(1),
        stdout: format!(
            "{}:2:1: warning: first [SC2006]\n{}:2:1: warning: second [SC2006]\n",
            staged.display(),
            nested.display()
        )
        .into_bytes(),
        stderr: Vec::new(),
    };

    let findings = handle_shellcheck_result(root, &[staged, nested], Ok(output)).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["scripts/nested/tracked.sh", "scripts/staged.sh"]
    );
}

#[test]
fn tracked_only_selects_indexed_shell_candidates() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = snapshot.paths_for(root);
    let config = config_with_rule("{trackedOnly: true}");
    let rule_filter = path_filter::RulePathFilter::new(root, &config, &config.rules[0]).unwrap();
    let options = tracked_options();
    let sources = snapshot.source_store_for(root);

    for selected in [
        select_shell_candidates(root, &options, &files, &[], &rule_filter, &snapshot, None),
        select_shell_candidates(
            root,
            &options,
            &files,
            &[],
            &rule_filter,
            &snapshot,
            Some(&sources),
        ),
    ] {
        assert_eq!(
            candidate_names(&selected, root),
            [
                "scripts/bash",
                "scripts/explicit",
                "scripts/modified.sh",
                "scripts/nested/tracked.sh",
                "scripts/posix",
                "scripts/staged.sh",
            ]
        );
    }
}

#[test]
fn default_selection_keeps_visible_untracked_and_explicit_files() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = snapshot.paths_for(root);
    let config = config_with_rule("{}");
    let rule_filter = path_filter::RulePathFilter::new(root, &config, &config.rules[0]).unwrap();
    let mut options = tracked_options();
    options.tracked_only = false;

    let selected =
        select_shell_candidates(root, &options, &files, &[], &rule_filter, &snapshot, None);
    let names = candidate_names(&selected, root);

    assert!(names.contains(&"scripts/untracked.sh".to_string()));
    assert!(names.contains(&"scripts/untracked-explicit".to_string()));
    assert!(names.contains(&"scripts/ignored-explicit".to_string()));
    assert!(!names.contains(&"scripts/unsupported".to_string()));
    assert!(!names.contains(&"scripts/ignored.sh".to_string()));
}

#[test]
fn tracked_only_uses_ignore_aware_visible_fallback_outside_git() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shellcheck-runner/tracked-only");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    std::fs::rename(root.join(".gitignore.fixture"), root.join(".gitignore")).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = snapshot.paths_for(root);
    let config = config_with_rule("{trackedOnly: true}");
    let rule_filter = path_filter::RulePathFilter::new(root, &config, &config.rules[0]).unwrap();

    let selected = select_shell_candidates(
        root,
        &tracked_options(),
        &files,
        &[],
        &rule_filter,
        &snapshot,
        None,
    );
    let names = candidate_names(&selected, root);

    assert!(names.contains(&"scripts/untracked.sh".to_string()));
    assert!(names.contains(&"scripts/untracked-explicit".to_string()));
    assert!(!names.contains(&"scripts/ignored.sh".to_string()));
    assert!(!names.contains(&"scripts/ignored-explicit".to_string()));
}

#[test]
fn standalone_and_prepared_dispatch_agree_on_tracked_only_findings() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let config = config_with_rule("{trackedOnly: true, shebangDirs: [scripts]}");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot =
        crate::codebase::ts_source::VisiblePathSnapshot::new_observed(root, Some(observer.clone()));
    let files = snapshot.paths_for(root);
    let initial_discovery_roots = observer.snapshot().work["discovery.roots"];

    let standalone = check(root, &config).unwrap();
    let prepared = filesystem_dispatch::run_filesystem_rules_with_config_and_snapshot(
        root, &config, &files, &snapshot,
    )
    .unwrap();

    assert_eq!(
        observer.snapshot().work["discovery.roots"],
        initial_discovery_roots
    );
    let standalone_files = standalone
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();
    let prepared_files = prepared
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();
    assert_eq!(standalone_files, prepared_files);
    if shellcheck_available() {
        assert_eq!(prepared_files, ["scripts/modified.sh"]);
    }
}
