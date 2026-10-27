use super::included_exclude::included_exclude;
use super::policy::{check_with_complete_names as policy_check, ExcludeEntry, Snapshot};
use super::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// Output cases moved to the cli_pkgcfg_snapshot modules.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-release-age-policy/fixture")
            .join(name),
    )
}

#[test]
fn conflict_entries_skip_exclude_drift() {
    let opts = Options {
        permanent_packages: vec![PermanentPackage {
            name: "acme-lib".to_string(),
            reason: "first-party".to_string(),
        }],
        ..Default::default()
    };
    let snapshot = Snapshot {
        exclude: vec![
            ExcludeEntry::Conflict,
            ExcludeEntry::Name("other".to_string()),
        ],
        cooldown: None,
        active_names: HashSet::new(),
        lockfile_keys: None,
    };
    let messages = policy_check(&opts, &snapshot, true)
        .into_iter()
        .map(|issue| issue.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("minimumReleaseAgeExclude")
                && !message.contains("other")),
        "{messages:?}"
    );
}

#[test]
fn unreadable_workspace_keeps_the_fallback_exclude_list() {
    let root = fixture("non-mapping-workspace");
    let path = root.join("pnpm-workspace.yaml");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let fallback = vec![ExcludeEntry::Name("kept".to_string())];
    let (entries, notes) = included_exclude(&root, &path, &sources, fallback);
    assert!(notes.is_empty());
    assert!(matches!(entries.first(), Some(ExcludeEntry::Name(name)) if name == "kept"));
    let (entries, notes) = included_exclude(
        Path::new("/repo"),
        Path::new("/"),
        &sources,
        vec![ExcludeEntry::Name("kept".to_string())],
    );
    assert!(notes.is_empty());
    assert!(matches!(entries.first(), Some(ExcludeEntry::Name(name)) if name == "kept"));
}
