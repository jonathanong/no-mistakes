use super::*;

#[test]
fn helper_review_regressions_preserve_call_identity_exports_and_live_captures() {
    let root = fixture("helper-tracing-review");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let paths = [
        "src/destructured-tag.mts",
        "src/import-equals.mts",
        "src/local-tag.mts",
        "src/named-tags.mts",
        "src/query.mts",
        "src/raw-assertion.ts",
        "src/raw-deleted.mts",
        "src/raw-reassigned.mts",
        "src/raw-shadowed.mts",
    ];
    for ignore in [false, true] {
        let policy = if ignore { "ignore" } else { "report" };
        let config = config_with_options(&format!(
            "importSpecifier: ''\nexecutorNames: [write, query]\ninclude: ['src/query.mts', 'src/import-equals.mts', 'src/destructured-tag.mts', 'src/local-tag.mts', 'src/named-tags.mts', 'src/raw-assertion.ts', 'src/raw-deleted.mts', 'src/raw-reassigned.mts', 'src/raw-shadowed.mts']\ntrustedSqlTags: [{{module: './tags.mjs', name: customQuery}}]\nunanalyzableSql: {policy}"
        ));
        let findings = check_with_files(&root, &config, &files).unwrap();
        for threads in [1, 3] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let parallel = pool.install(|| check_with_files(&root, &config, &files).unwrap());
            assert_eq!(
                parallel, findings,
                "helper projection differs with {threads} workers"
            );
        }
        let expected = paths
            .iter()
            .flat_map(|path| {
                std::fs::read_to_string(root.join(path))
                    .unwrap()
                    .lines()
                    .enumerate()
                    .filter_map(|(index, line)| {
                        (line.contains("// finding:")
                            || (!ignore && line.contains("// unanalyzable:")))
                        .then_some((path.to_string(), index + 1))
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            findings
                .iter()
                .map(|finding| (finding.file.clone(), finding.line))
                .collect::<Vec<_>>(),
            expected,
            "{findings:#?}"
        );
    }
}

#[test]
fn executor_free_root_still_reports_imported_executor_through_cyclic_reexports() {
    let root = fixture("helper-tracing-reachability");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let config = config_with_options(
        "importSpecifier: '@app/db'\ninclude: ['src/helper.mts']\nunanalyzableSql: ignore",
    );
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/helper.mts");
    assert_eq!(findings[0].line, 3);
}

#[test]
fn repeated_conditional_builder_updates_preserve_unproven_sql() {
    let root = fixture("helper-tracing-joined-binding");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    for (policy, expected) in [("report", 1), ("ignore", 0)] {
        let config = config_with_options(&format!(
            "importSpecifier: '@app/db'\ninclude: ['src/query.mts']\nunanalyzableSql: {policy}"
        ));
        let findings = check_with_files(&root, &config, &files).unwrap();
        assert_eq!(findings.len(), expected, "{policy}: {findings:#?}");
        if let Some(finding) = findings.first() {
            assert_eq!(finding.file, "src/query.mts");
            assert_eq!(finding.line, 15);
        }
    }
}

#[test]
fn shared_container_snapshots_preserve_both_indexed_findings() {
    let root = fixture("helper-tracing-values-cow");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let config = config_with_options(
        "importSpecifier: '@app/db'\ninclude: ['src/query.mts']\nunanalyzableSql: report",
    );
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .all(|finding| finding.file == "src/query.mts"));
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [8, 9]
    );
}
