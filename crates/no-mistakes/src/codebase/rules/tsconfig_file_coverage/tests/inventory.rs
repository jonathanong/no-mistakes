fn temp_root() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(&dir.path().canonicalize().unwrap());
    (dir, root)
}

fn write(root: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn missing_tsconfig_is_silent() {
    let (_dir, root) = temp_root();
    let ts = write(&root, "index.ts", "export {}\n");
    let findings = check_with_files(&root, &config("{}"), &[ts]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_tsconfig_still_rejects_malformed_options() {
    let (_dir, root) = temp_root();
    let files = vec![
        write(&root, "index.ts", "export {}\n"),
        write(&root, "helper.json", "{}\n"),
    ];
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
    let (_dir, root) = temp_root();
    let files = vec![
        write(&root, "tsconfig.json", "{ \"files\": [\"src/index.ts\"] }\n"),
        write(&root, "src/index.ts", "export {}\n"),
        write(&root, "src/extra.ts", "export {}\n"),
        write(&root, "scripts/scratch.ts", "export {}\n"),
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
    let (_dir, root) = temp_root();
    let files = vec![
        write(
            &root,
            "tsconfig.json",
            "{ \"files\": [\"scripts/generate.ts\"], \"include\": [\"src\"] }\n",
        ),
        write(&root, "src/index.ts", "export {}\n"),
        write(&root, "scripts/generate.ts", "export {}\n"),
        write(&root, "orphan.ts", "export {}\n"),
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
    let (_dir, root) = temp_root();
    let files = vec![
        write(&root, "index.ts", "export {}\n"),
        write(&root, "tsconfig.dependency-cruiser.json", "{}\n"),
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
    let (_dir, root) = temp_root();
    let files = vec![
        write(&root, "tsconfig.json", "{ \"include\": [\"index.ts\"] }\n"),
        write(&root, "index.ts", "export {}\n"),
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
    let (_dir, root) = temp_root();
    let files = vec![
        write(&root, "tsconfig.json", "{ \"include\": [\"index.ts\"] }\n"),
        write(&root, "index.ts", "export {}\n"),
        write(&root, "tsconfig.dependency-cruiser.json", "[]\n"),
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
    let (_dir, root) = temp_root();
    let files = vec![
        write(&root, "tsconfig.json", "{ \"include\": [\"index.ts\"] }\n"),
        write(&root, "index.ts", "export {}\n"),
        write(&root, "foo.json", "{ \"include\": [\"index.ts\"] }\n"),
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
