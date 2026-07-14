use super::*;

fn findings(root: &Path) -> Vec<UniqueExportFinding> {
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );
    analyze_project_with_facts(root, None, None, &facts).unwrap()
}

#[test]
fn ignored_ancestor_package_does_not_enable_nextjs_exemptions() {
    let fixture =
        crate::test_support::materialize_gitignore_fixture("unique-exports-ignored-next-package");

    let findings = findings(fixture.path());

    assert!(findings
        .iter()
        .any(|finding| finding.export_name == "metadata"));
}

#[test]
fn tracked_ignored_ancestor_package_enables_nextjs_exemptions() {
    let fixture =
        crate::test_support::materialize_gitignore_fixture("unique-exports-ignored-next-package");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    crate::test_support::git_add_force(fixture.path(), &["package.json"]);

    let findings = findings(fixture.path());

    assert!(!findings
        .iter()
        .any(|finding| finding.export_name == "metadata"));
}
