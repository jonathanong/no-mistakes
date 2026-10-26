use super::rule_docs_helpers::{read, repo_root};

#[test]
fn lint_suppression_docs_keep_the_explicit_configuration_contract() {
    let root = repo_root();
    let body = read(&root.join("docs/rules/forbidden-lint-suppressions.md"));
    for marker in [
        "files:",
        "ruleIds:",
        "rulePrefixes:",
        "directives:",
        "banBlanketDisable:",
        "message:",
        "repository-relative",
        "`[]` selects no directives",
        "`@ts-ignore` and `@ts-expect-error` are opt-in",
        "no-mistakes-disable-file forbidden-lint-suppressions",
        "no-mistakes-disable-line forbidden-lint-suppressions",
        "no-mistakes-disable-next-line forbidden-lint-suppressions",
    ] {
        assert!(
            body.contains(marker),
            "missing rule documentation: {marker}"
        );
    }
    let node = read(&root.join("docs/node-api.md"));
    for marker in [
        "ForbiddenLintSuppressionsOptions",
        "ForbiddenLintSuppressionDirective",
        "checkJson",
    ] {
        assert!(node.contains(marker), "missing Node contract: {marker}");
    }
}
