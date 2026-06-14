use super::*;

#[test]
fn collect_swift_facts_returns_empty_without_configured_packages() {
    let root = Path::new("/repo");
    assert!(collect_swift_facts(root, &[], &[]).files.is_empty());
}

#[test]
fn collect_swift_facts_returns_empty_when_packages_do_not_parse() {
    let root = Path::new("/repo");
    let files = vec![PathBuf::from("/repo/Client/Sources/App/App.swift")];
    assert!(collect_swift_facts(root, &files, &["Client".to_string()])
        .files
        .is_empty());
}
