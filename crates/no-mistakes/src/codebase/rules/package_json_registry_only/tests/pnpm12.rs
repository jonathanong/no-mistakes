use super::*;

#[test]
fn pnpm12_env_tarball_is_reported() {
    // A first-document reader reports this package. A last-document reader misses it.
    let root = pnpm12_rule_root("pnpm12-env-tarball");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("config-plugin@1.0.0"),
        "{findings:?}"
    );
    assert!(findings[0].message.contains("tarball"), "{findings:?}");
    assert!(
        !findings[0].message.contains("react@19.0.0"),
        "{findings:?}"
    );
}

#[test]
fn issue_1035_exotic_tarball_in_the_project_document_is_reported() {
    let root = issue_1035_root("exotic");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("is-buffer@1.1.6"),
        "{findings:?}"
    );
    assert!(findings[0].message.contains("tarball"), "{findings:?}");
    assert!(!findings[0].message.contains("@pnpm/exe"), "{findings:?}");
}

#[test]
fn issue_1035_registry_lockfile_has_no_findings() {
    for variant in ["two-doc", "single-doc"] {
        let findings = check_with_files(
            &issue_1035_root(variant),
            &config_with_options("lockfile: pnpm-lock.yaml"),
            &[],
        )
        .unwrap();
        assert!(findings.is_empty(), "{variant}: {findings:?}");
    }
}

#[test]
fn pnpm12_duplicate_key_is_reported_once() {
    // The same non-registry key is in both documents. Diff identity stays a set;
    // this rule must not emit two findings for one key and resolution.
    let root = pnpm12_rule_root("pnpm12-duplicate-tarball");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("config-plugin@1.0.0"),
        "{findings:?}"
    );
    assert!(findings[0].message.contains("tarball"), "{findings:?}");
}

#[test]
fn pnpm12_project_git_resolution_is_reported() {
    // A first-document reader misses this package because it only sees the pnpm pin.
    let root = pnpm12_rule_root("pnpm12-project-git");
    let config = config_with_options("lockfile: pnpm-lock.yaml");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("github.com/org/repo"),
        "{findings:?}"
    );
    assert!(findings[0].message.contains("repo"), "{findings:?}");
    assert!(!findings[0].message.contains("pnpm@12.3.4"), "{findings:?}");
}
