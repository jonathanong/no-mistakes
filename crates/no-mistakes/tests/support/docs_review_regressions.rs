use std::path::PathBuf;

fn read_root(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read_to_string(root.join(path)).unwrap()
}

#[test]
fn reviewed_commands_and_rule_options_stay_accurate() {
    let agent_guide = read_root("docs/agent-guide.md");
    assert!(agent_guide.contains("ci topology-impact --base <ref> --head HEAD --entry-workflow"));
    assert!(!agent_guide.contains("ci topology-impact --base <ref> --head HEAD --format"));

    let reviewed_rule_options = [
        ("server-route-client-boundary.md", &["`excludes`"][..]),
        ("rust-no-inline-allows.md", &["`roots`", "`excludes`"][..]),
        ("rust-no-inline-tests.md", &["`roots`", "`excludes`"][..]),
        (
            "shellcheck-runner.md",
            &["`shellFiles`", "`shebangDirs`", "`shellcheck.severity`"][..],
        ),
        (
            "csharp-max-lines-per-file.md",
            &["`srcMax`", "`testMax`", "`excludes`", "`testRoots`"][..],
        ),
        (
            "package-json-required-fields.md",
            &[
                "`private`",
                "`type`",
                "`license`",
                "`requireScopedName`",
                "`unscopedNameExceptions`",
                "`mainWhenFileExists`",
            ][..],
        ),
        (
            "no-git-identity-mutation.md",
            &["`excludePaths`", "`conditionallyAllowedWorkflows`"][..],
        ),
        (
            "require-test-per-subdir.md",
            &["`roots`", "`testGlob`", "`excludeDirs`", "`directChild`"][..],
        ),
        (
            "required-local-docs.md",
            &[
                "`roots`",
                "`requiredFile`",
                "`codeExtensions`",
                "`testExcludePatterns`",
            ][..],
        ),
        (
            "rust-max-lines-per-file.md",
            &["`srcMax`", "`testMax`", "`roots`", "`excludes`"][..],
        ),
        (
            "tsconfig-alias-folder-mapping.md",
            &["`checkExists`", "`false`"][..],
        ),
        (
            "vitest-test-correspondence.md",
            &[
                "`scopes`",
                "`testExtensions`",
                "`testsDir`",
                "`direction`",
                "`stemSuffixesToStrip`",
                "`duplicateStemGroup`",
            ][..],
        ),
        ("markdown-link-display-text.md", &["`extensions`"][..]),
        (
            "nextjs-redirect-destinations.md",
            &["`configPath`", "`appRoot`", "`includeRewrites`"][..],
        ),
        (
            "integration-test-no-mocks.md",
            &["`forbiddenCalls`", "`forbiddenModules`"][..],
        ),
        (
            "markdown-child-links.md",
            &[
                "`groups`",
                "`parents`",
                "`children`",
                "`requireWholeFile`",
                "`countCanonicalHtmlListItems`",
            ][..],
        ),
        (
            "package-json-nested-workspace-coverage.md",
            &["`roots`", "`dependencyNamePrefixes`", "`dependencyFields`"][..],
        ),
        (
            "package-json-workspace-coverage.md",
            &["`packageRoots`", "`allowlist`", "`requireNamedPackage`"][..],
        ),
        (
            "postgres-no-generated-column-writes.md",
            &[
                "`sqlInclude`",
                "`include`",
                "`importSpecifier`",
                "`executorNames`",
                "`extraGeneratedColumns`",
            ][..],
        ),
        (
            "require-storybook-stories.md",
            &[
                "`include`",
                "`includeAllReactNamedExports`",
                "`includeAllReactDefaultExports`",
                "`requiredProps`",
                "`allowComponents`",
                "`allowFiles`",
                "`allowColocatedTests`",
                "`ignoreIndexAndPrivateFiles`",
            ][..],
        ),
        (
            "required-doc-section.md",
            &["`glob`", "`requiredHeading`"][..],
        ),
        ("shellcheck-runner.md", &["`skillsLockfile`"][..]),
        (
            "test-no-dependency-pins.md",
            &["`include`", "`patterns`", "reason:", "regex:"][..],
        ),
        (
            "workflow-topology-policy.md",
            &[
                "`requiredJobs`",
                "`forbiddenJobs`",
                "`requiredDirectEdges`",
                "`forbiddenDirectEdges`",
                "`requiredTransitiveEdges`",
                "`forbiddenTransitiveEdges`",
                "`requiredArtifactEdges`",
                "`exactFanIns`",
                "`exactCallerJobs`",
                "`stepOrders`",
                "`unlockedWorkflowReasons`",
            ][..],
        ),
    ];

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/rules");
    for (file, options) in reviewed_rule_options {
        let body = std::fs::read_to_string(root.join(file)).unwrap();
        for option in options {
            assert!(body.contains(option), "{file} must document {option}");
        }
    }
}

#[test]
fn reviewed_caveats_stay_explicit() {
    assert!(
        read_root("docs/rules/nextjs-no-caching.md").contains("There are no rule-local options.")
    );
    assert!(read_root("docs/rules/nextjs-no-api-routes.md").contains("`app/users/page.tsx`"));
    assert!(read_root("docs/rules/package-json-required-fields.md")
        .contains("not an additional option"));
    assert!(
        read_root("docs/rules/postgres-no-generated-column-writes.md")
            .contains("There are no direct\n`schema` or `embedded` options.")
    );
    assert!(read_root("docs/rules/test-no-dependency-pins.md")
        .contains("There is no user-facing `defaultInclude` option."));

    let architecture = read_root("docs/architecture.md");
    assert!(architecture.contains("different\nruntime environments"));
    assert!(architecture.contains("unexpected base"));
}
