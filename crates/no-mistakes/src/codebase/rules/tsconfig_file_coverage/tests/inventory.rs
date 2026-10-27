#[test]
fn missing_tsconfig_is_silent() {
    let root = fixture("missing-tsconfig");
    let ts = root.join("index.ts");
    let findings = check_with_files(&root, &config("{}"), &[ts]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_tsconfig_still_rejects_malformed_options() {
    let root = fixture("malformed-options");
    let files = vec![root.join("index.ts"), root.join("helper.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
allow:
  - path: /index.ts
    reason: must not rewrite
  - path: index.ts
    reason: ""
auxiliaryConfigs:
  - path: helper.json
    reason: misnamed helper
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("must be a repository-relative path without parent traversals")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("empty reason")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("basename must be")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("not covered by any tsconfig")),
        "{findings:?}"
    );
}

#[test]
fn source_include_keeps_root_tsconfig_membership() {
    let root = fixture("source-include");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("src/index.ts"),
        root.join("src/extra.ts"),
        root.join("scripts/scratch.ts"),
    ];
    let mut cfg = config("{}");
    cfg.rules[0].include = vec!["src/**/*.ts".into()];
    let findings = check_with_files(&root, &cfg, &files).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.file.contains("extra.ts")
                && finding.message.contains("not covered by any tsconfig")),
        "{findings:?}"
    );
    assert!(
        findings.iter().all(|finding| {
            !finding.file.contains("index.ts") && !finding.file.contains("scratch.ts")
        }),
        "{findings:?}"
    );
}

#[test]
fn files_and_include_union_covers_both_sets() {
    let root = fixture("files-and-include");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("src/index.ts"),
        root.join("scripts/generate.ts"),
        root.join("orphan.ts"),
    ];
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.file.contains("orphan.ts")),
        "{findings:?}"
    );
    assert!(
        findings.iter().all(|finding| {
            !finding.file.contains("index.ts") && !finding.file.contains("generate.ts")
        }),
        "{findings:?}"
    );
}

#[test]
fn auxiliary_only_inventory_leaves_sources_uncovered() {
    let root = fixture("auxiliary-only");
    let files = vec![
        root.join("index.ts"),
        root.join("tsconfig.dependency-cruiser.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: resolver config
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.file.contains("index.ts")
                && finding.message.contains("not covered by any tsconfig")),
        "{findings:?}"
    );
}

#[test]
fn missing_auxiliary_source_is_not_a_json_object() {
    let root = fixture("missing-auxiliary");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("index.ts"),
        root.join("tsconfig.dependency-cruiser.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: missing from disk
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not a JSON object")),
        "{findings:?}"
    );
}

#[test]
fn auxiliary_array_json_is_not_an_object() {
    let root = fixture("auxiliary-array");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("index.ts"),
        root.join("tsconfig.dependency-cruiser.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: array document
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not a JSON object")),
        "{findings:?}"
    );
}

#[test]
fn tracked_non_tsconfig_auxiliary_is_still_read() {
    let root = fixture("tracked-auxiliary");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("index.ts"),
        root.join("foo.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: foo.json
    reason: misnamed helper
    requiredBasename: foo.json
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("must not declare files, include, exclude, or references")),
        "{findings:?}"
    );
}
