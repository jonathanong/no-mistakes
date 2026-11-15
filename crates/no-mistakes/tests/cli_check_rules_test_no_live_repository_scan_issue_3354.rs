use std::collections::BTreeSet;

#[test]
fn owned_root_paths_options_and_globs_check_json_snapshot() {
    let root =
        crate::check_json_snapshot::rules_path("test-no-live-repository-scan", "issue-3354", "");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_issue_3354",
        &[("ownership", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    assert_eq!(reports["ownership"]["exitCode"], 1);
    let report = &reports["ownership"]["report"];
    assert!(
        report["advisories"].as_array().unwrap().is_empty(),
        "{report}"
    );
    let findings = report["rules"].as_array().unwrap();
    let actual: BTreeSet<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], "test-no-live-repository-scan");
            (
                finding["file"].as_str().unwrap(),
                finding["sourceLine"].as_str().unwrap(),
            )
        })
        .collect();
    let expected = BTreeSet::from([
        (
            "tests/unsafe-globs.test.mts",
            "import.meta.glob('**/*.test.mts', { cwd: root });",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "import.meta.globEager('**/*.test.mts', { cwd: root });",
        ),
        (
            "tests/unsafe-options.test.mts",
            "readdirSync(unsafeDirect.root);",
        ),
        (
            "tests/unsafe-options.test.mts",
            "readdirSync(mutatedDirect.root);",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, emptyName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, overLimitName));",
        ),
        ("tests/unsafe-eval.test.mts", "readdirSync(root);"),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, unknownChild));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, name));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, absoluteName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, mutableName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, unknownName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, mixedName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, reassignedName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, spreadName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, windowsName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, uncName));",
        ),
        (
            "tests/unsafe-joins.test.mts",
            "readdirSync(join(root, longName));",
        ),
        (
            "src/overridden-options.mts",
            "return readdirSync(options.root);",
        ),
        (
            "src/opaque-options.mts",
            "return readdirSync(options.root);",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "globSync('**/*.test.mts', { cwd: process.cwd() });",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "fsGlobSync('**/*.test.mts');",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "fastGlobSync('**/*.test.mts');",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "globSync(unknownPattern, { cwd: root });",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "globSync('/outside/**/*.test.mts', { cwd: root });",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "globSync('../**/*.test.mts', { cwd: root });",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "globSync('**/*.test.mts', opaqueOptions);",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "globSync('**/*.test.mts', { cwd: root, ...opaqueOptions });",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "globSync('**/*.test.mts', { cwd: root, cwd: process.cwd() });",
        ),
        (
            "tests/unsafe-globs.test.mts",
            "readdirSync('./src', { cwd: root });",
        ),
    ]);
    assert_eq!(findings.len(), expected.len(), "{report}");
    assert_eq!(actual, expected, "{report}");
}
