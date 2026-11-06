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
