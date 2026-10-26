use super::*;

#[test]
fn pass_fixture_produces_no_findings() {
    let root = fixture("pass");
    let config = config_with_options("{}");
    let files = vec![root.join("package.json")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn fail_fixture_produces_findings() {
    let root = fixture("fail");
    let config = config_with_options("{}");
    let files = vec![root.join("package.json")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(!findings.is_empty(), "expected at least one finding");
    assert!(
        findings.iter().any(|f| f.message.contains("file:")),
        "expected file: finding"
    );
}

#[test]
fn unreadable_package_json_is_unanalyzable() {
    let root = fixture("missing-input");
    let findings = check_package_json(&root.join("missing/package.json"), &root);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn configured_missing_lockfile_is_unanalyzable() {
    let root = fixture("missing-input");
    let findings = check_with_files(
        &root,
        &config_with_options("lockfile: nonexistent.yaml"),
        &[],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn invalid_json_in_package_json_is_unanalyzable() {
    let root = fixture("unanalyzable");
    let findings = check_package_json(&root.join("packages/broken/package.json"), &root);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn issue_1035_unreadable_lockfile_does_not_pass() {
    let malformed = check_with_files(
        &issue_1035_root("malformed"),
        &config_with_options("lockfile: pnpm-lock.yaml"),
        &[],
    )
    .unwrap();
    assert!(
        malformed[0].message.contains("could not be parsed"),
        "{malformed:?}"
    );
    let non_env = check_with_files(
        &issue_1035_root("non-env"),
        &config_with_options("lockfile: pnpm-lock.yaml"),
        &[],
    )
    .unwrap();
    assert!(non_env[0].message.contains("non-env"), "{non_env:?}");
}

#[test]
fn configured_lockfile_is_required_per_selected_project_and_routes_policy() {
    let root = fixture("project-lockfile-scope");
    let files = [
        root.join("web/package.json"),
        root.join("admin/package.json"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for (name, missing) in [
        ("both-projects.yml", 1),
        ("web-only.yml", 0),
        ("excluded-admin.yml", 0),
    ] {
        let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
        for (policy, findings, advisories) in [
            ("report", missing, 0),
            ("advise", 0, missing),
            ("ignore", 0, 0),
        ] {
            config.rules[0].options =
                serde_yaml::from_str(&format!("lockfile: pnpm-lock.yaml\nunanalyzable: {policy}"))
                    .unwrap();
            let diagnostics =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                    .unwrap();
            assert_eq!(diagnostics.findings.len(), findings, "{name}: {policy}");
            assert_eq!(diagnostics.advisories.len(), advisories, "{name}: {policy}");
            for finding in diagnostics.findings.iter().chain(&diagnostics.advisories) {
                assert_eq!(finding.file, "admin/pnpm-lock.yaml");
                assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
            }
        }
    }
    // Selecting or excluding projects never repeats the same physical read.
    assert_eq!(sources.physical_read_count(), 4);
}
