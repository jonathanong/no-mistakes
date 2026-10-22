use super::super::check_with_files;
use super::{config, fixture_root, package_files};

#[test]
fn duplicate_catalog_finding_stays_inside_the_rule_scope() {
    let root = fixture_root("catalog-duplicate");
    let files = package_files(
        &root,
        &[
            "package.json",
            "pnpm-workspace.yaml",
            "pnpm/catalog.yaml",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );
    let mut scoped = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    scoped.rules[0].include = vec!["packages/**".to_string()];
    let findings = check_with_files(&root, &scoped, &files).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.file != "pnpm-workspace.yaml"),
        "{findings:?}"
    );

    let walked = fixture_root("catalog-workspace");
    let walked_files = package_files(
        &walked,
        &[
            "package.json",
            "pnpm-workspace.yaml",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );
    let mut walked_config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    walked_config.rules[0].include = vec!["packages/**".to_string()];
    let findings = check_with_files(&walked, &walked_config, &walked_files).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.target.as_deref() == Some("@acme/secret")),
        "{findings:?}"
    );
}
