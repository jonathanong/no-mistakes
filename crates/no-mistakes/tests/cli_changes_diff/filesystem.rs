use super::fixture;
use no_mistakes::changes::{self, ChangesDiffArgs};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{symlink, PermissionsExt};

fn materialized_filesystem_fixture() -> (tempfile::TempDir, ChangesDiffArgs) {
    let root = crate::saved_fixture::materialize_path(&fixture().join("filesystem"));
    let args = ChangesDiffArgs {
        base_root: root.path().join("base"),
        head_root: root.path().join("head"),
        config: None,
    };
    (root, args)
}

#[test]
fn changes_diff_reports_paths_with_invalid_utf8_without_lossy_classification() {
    let (_root, args) = materialized_filesystem_fixture();
    let name = OsString::from_vec(b"invalid-\xff.txt".to_vec());
    fs::rename(
        args.head_root.join("redirect.txt"),
        args.head_root.join(name),
    )
    .unwrap();
    let report = changes::diff(&args).unwrap();
    assert!(!report.complete);
    assert_eq!(report.categories["files"].changed, ["edited.txt"]);
    assert_eq!(report.categories["files"].removed, ["redirect.txt"]);
    assert!(report.unclassified.is_empty());
    assert_eq!(report.reasons.len(), 1);
    assert!(report.reasons[0].contains("non-UTF-8 head path"));
}

#[test]
fn changes_diff_compares_links_by_target_without_reading_through_them() {
    let root = crate::saved_fixture::materialize_path(&fixture().join("symlinks"));
    let report = changes::diff(&ChangesDiffArgs {
        base_root: root.path().join("base"),
        head_root: root.path().join("head"),
        config: None,
    })
    .unwrap();
    assert!(report.complete, "{:?}", report.reasons);
    assert!(report.reasons.is_empty());
    assert_eq!(report.categories["docs"].changed, ["docs/guide.md"]);
    assert_eq!(report.categories["skills"].changed, [".claude/skills/y"]);
    assert!(report.unclassified.is_empty());
}

#[test]
fn changes_diff_treats_a_dangling_link_with_a_new_target_as_a_path_change() {
    let root = crate::saved_fixture::materialize_path(&fixture().join("symlinks"));
    let head = root.path().join("head");
    fs::remove_file(head.join("other/gone")).unwrap();
    symlink("elsewhere", head.join("other/gone")).unwrap();
    let report = changes::diff(&ChangesDiffArgs {
        base_root: root.path().join("base"),
        head_root: head,
        config: None,
    })
    .unwrap();
    assert!(report.complete);
    assert_eq!(report.unclassified, ["other/gone"]);
}

#[test]
fn changes_diff_reports_read_failures_on_either_side_and_keeps_other_changes() {
    let (_root, args) = materialized_filesystem_fixture();
    let base = args.base_root.join("restricted/base.txt");
    let head = args.head_root.join("restricted/head.txt");
    fs::set_permissions(&base, fs::Permissions::from_mode(0o000)).unwrap();
    fs::set_permissions(&head, fs::Permissions::from_mode(0o000)).unwrap();
    let result = changes::diff(&args);
    fs::set_permissions(base, fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(head, fs::Permissions::from_mode(0o600)).unwrap();
    let report = result.unwrap();
    assert!(!report.complete);
    assert_eq!(report.categories["files"].changed, ["edited.txt"]);
    assert!(report.unclassified.is_empty());
    assert_eq!(report.reasons.len(), 2);
    assert!(report.reasons[0].contains(r#"cannot read base file "restricted/base.txt""#));
    assert!(report.reasons[1].contains(r#"cannot read head file "restricted/head.txt""#));
}

#[test]
fn changes_diff_does_not_infer_removals_from_an_unreadable_subtree() {
    let (_root, mut args) = materialized_filesystem_fixture();
    let blocked = args.head_root.join("restricted");
    let original = fs::metadata(&blocked).unwrap().permissions();
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).unwrap();
    let result_with_default_config = changes::diff(&args);
    args.config = Some("missing.yml".into());
    let result_with_missing_config = changes::diff(&args);
    fs::set_permissions(blocked, original).unwrap();
    let partial = result_with_default_config.unwrap();
    assert!(!partial.complete);
    assert_eq!(partial.categories["files"].changed, ["edited.txt"]);
    assert!(partial.unclassified.is_empty());
    assert_eq!(partial.reasons.len(), 1);
    assert!(partial.reasons[0].contains("cannot enumerate head root"));
    assert!(partial.reasons[0].contains("restricted"));
    let incomplete = result_with_missing_config.unwrap();
    assert!(!incomplete.complete);
    assert!(incomplete.unclassified.is_empty());
    assert_eq!(incomplete.reasons, partial.reasons);
}
