use super::*;

#[test]
fn shared_suppression_only_reads_repo_relative_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("safe.ts"),
        "// no-mistakes-disable-file my-rule\n",
    )
    .unwrap();

    let mut findings = vec![
        RuleFinding {
            rule: "my-rule".to_string(),
            file: "safe.ts".to_string(),
            line: 1,
            message: "safe".to_string(),
            import: None,
            target: None,
        },
        RuleFinding {
            rule: "my-rule".to_string(),
            file: "../safe.ts".to_string(),
            line: 1,
            message: "parent".to_string(),
            import: None,
            target: None,
        },
        RuleFinding {
            rule: "my-rule".to_string(),
            file: root.join("safe.ts").display().to_string(),
            line: 1,
            message: "absolute".to_string(),
            import: None,
            target: None,
        },
    ];

    suppress_rule_findings(root, &mut findings);

    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.clone())
        .collect();
    assert_eq!(files.len(), 2);
    assert_eq!(
        files,
        vec![
            "../safe.ts".to_string(),
            root.join("safe.ts").display().to_string()
        ]
    );
}

#[test]
fn shared_suppression_keeps_findings_when_root_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("missing");
    let mut findings = vec![RuleFinding {
        rule: "my-rule".to_string(),
        file: "safe.ts".to_string(),
        line: 1,
        message: "safe".to_string(),
        import: None,
        target: None,
    }];

    suppress_rule_findings(&root, &mut findings);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "safe.ts");
}

#[test]
#[cfg(unix)]
fn shared_suppression_does_not_follow_symlinks_outside_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(outside.path(), "// no-mistakes-disable-file my-rule\n").unwrap();
    std::os::unix::fs::symlink(outside.path(), root.join("link.ts")).unwrap();

    let mut findings = vec![RuleFinding {
        rule: "my-rule".to_string(),
        file: "link.ts".to_string(),
        line: 1,
        message: "symlink".to_string(),
        import: None,
        target: None,
    }];

    suppress_rule_findings(root, &mut findings);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "link.ts");
}

#[test]
fn shared_suppression_only_reads_regular_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir(root.join("not-a-file.ts")).unwrap();

    let mut findings = vec![RuleFinding {
        rule: "my-rule".to_string(),
        file: "not-a-file.ts".to_string(),
        line: 1,
        message: "directory".to_string(),
        import: None,
        target: None,
    }];

    suppress_rule_findings(root, &mut findings);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "not-a-file.ts");
}

#[test]
#[cfg(unix)]
fn shared_suppression_reuses_the_lexical_symlink_source_slot() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let target = root.join("target.ts");
    let link = root.join("link.ts");
    std::fs::write(&target, "// no-mistakes-disable-file my-rule\n").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let sources = super::source_store_for_files(std::slice::from_ref(&link));
    assert!(sources.read_path(&link).is_ok());

    let mut findings = vec![RuleFinding {
        rule: "my-rule".to_string(),
        file: "link.ts".to_string(),
        line: 1,
        message: "symlink".to_string(),
        import: None,
        target: None,
    }];
    suppress_rule_findings_with_sources(root, &mut findings, &sources);

    assert!(findings.is_empty());
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn shared_suppression_uses_frozen_source_after_file_deletion() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let path = root.join("safe.ts");
    std::fs::write(&path, "// no-mistakes-disable-file my-rule\n").unwrap();
    let sources = super::source_store_for_files(std::slice::from_ref(&path));
    assert!(sources.read_path(&path).is_ok());
    std::fs::remove_file(&path).unwrap();

    let mut findings = vec![RuleFinding {
        rule: "my-rule".to_string(),
        file: "safe.ts".to_string(),
        line: 1,
        message: "deleted".to_string(),
        import: None,
        target: None,
    }];
    suppress_rule_findings_with_sources(root, &mut findings, &sources);

    assert!(findings.is_empty());
    assert_eq!(sources.physical_read_count(), 1);
}
