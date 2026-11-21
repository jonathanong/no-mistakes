use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn vitest_project_policy_json_matches_snapshot() {
    let mut cases = [
        "pass",
        "declared",
        "declared-filter",
        "helper-glob-skip",
        "helper-literal-file-brackets",
        "helper-literal-file-open-bracket",
        "helper-literal-dir-brackets",
        "helper-literal-dir-open-bracket",
        "declared-source-errors",
        "declared-line",
        "declared-next-line",
        "declared-file",
        "literal-forms",
        "custom-advisory",
        "excluded-root",
        "missing",
        "mismatch",
        "nonliteral",
        "helper",
        "required-forbidden",
        "suppression",
        "suppression-incomplete",
        "line-suppression",
        "next-line-suppression",
        "spread",
        "dynamic-project",
        "project-hole",
        "root-mismatch",
        "undeclared-helper",
        "spread-before",
        "spread-after",
        "arbitrary-call",
        "mutable-binding",
        "wrapper-alias",
        "helper-chain",
        "helper-cycle",
        "root-missing",
        "reexport-undeclared",
        "signed-literal",
        "mutated-object",
        "unknown-inheritance",
        "missing-test",
        "missing-source",
        "malformed-source",
        "nonliteral-root",
        "missing-root-test",
        "projects-nonarray",
        "missing-helper",
        "malformed-helper",
    ]
    .into_iter()
    .map(|name| {
        let root = rules_path("vitest-project-policy", "fixture", name);
        (name, root.clone(), root.join(".no-mistakes.yml"))
    })
    .collect::<Vec<_>>();
    for name in [
        "effective",
        "combined",
        "array-value",
        "object-value",
        "empty-key",
        "dotted-key",
        "invalid-kind",
        "removed-required",
        "invalid-glob",
        "missing-declared-sources",
        "malformed-declared-sources",
    ] {
        let root = rules_path("vitest-project-policy", "fixture", "declared");
        cases.push((name, root.clone(), root.join(format!("{name}.yml"))));
    }
    for name in [
        "no-config",
        "array-required",
        "object-required",
        "empty-property",
        "dotted-property",
        "parity-length",
        "unknown-option",
        "suffix-undeclared",
        "wrapper-type-only",
        "helper-type-only",
        "update-mutation",
        "delete-mutation",
        "namespace-undeclared",
        "namespace-missing",
        "namespace-malformed",
        "namespace-known",
        "namespace-empty",
        "namespace-partial",
        "namespace-star-unknown",
        "namespace-star-known",
        "namespace-named-star",
        "namespace-mutable",
        "namespace-destructured",
    ] {
        let root = rules_path("vitest-project-policy", "fixture", "options-errors");
        cases.push((name, root.clone(), root.join(format!("{name}.yml"))));
    }
    for name in [
        "repeated-helper",
        "local-cycle",
        "unary-string",
        "unary-operator",
        "nonfinite-number",
        "unary-nonfinite",
        "wrapper-no-arguments",
        "wrapper-spread-argument",
        "wrapper-multiple-arguments",
        "missing-member",
        "missing-namespace-member",
        "known-namespace-member",
        "unknown-expression",
        "computed-property",
        "method-property",
        "opaque-array-spread",
        "only-type-reexports",
        "unsupported-export-declarations",
        "unresolved-local-export",
        "ambiguous-star-exports",
        "opaque-star-exports",
    ] {
        let root = rules_path("vitest-project-policy", "fixture", "lexical-boundaries");
        cases.push((name, root.clone(), root.join(format!("{name}.yml"))));
    }
    for name in [
        "no-projects",
        "neither-pair",
        "root-nonliteral",
        "unknown-inherits",
        "opaque-inherits",
        "root-only-inheritance",
        "root-only-inheritance-explicit",
    ] {
        let root = rules_path("vitest-project-policy", "fixture", "policy-boundaries");
        cases.push((name, root.clone(), root.join(format!("{name}.yml"))));
    }
    for name in [
        "exclude-empty-array",
        "exclude-missing-glob",
        "exclude-present",
        "exclude-omitted",
        "exclude-no-test",
        "exclude-explicit-empty",
        "exclude-local-binding",
        "exclude-helper",
        "exclude-proven-spread",
        "exclude-partial-spread",
        "exclude-dynamic",
        "exclude-dynamic-config",
        "exclude-dynamic-projects",
        "exclude-string-project",
        "exclude-project-present",
        "exclude-project-silent",
        "exclude-project-empty",
        "exclude-project-override",
        "exclude-project-inherit",
        "exclude-option-off",
        "exclude-non-array",
        "exclude-empty-glob",
        "exclude-conditional-spread",
        "exclude-conditional-exclude",
        "exclude-conditional-proven",
        "exclude-conditional-unknown",
        "exclude-ternary-complete-or-empty",
        "exclude-ternary-partial-or-empty",
        "exclude-ternary-complete-or-complete",
        "exclude-ternary-complete-or-partial",
        "exclude-ternary-complete-or-call",
        "exclude-ternary-partial-or-call",
        "exclude-and-complete",
        "exclude-and-partial",
        "exclude-earlier-then-partial",
        "exclude-call-make-extra",
        "exclude-computed-or-empty",
        "exclude-computed-both",
        "exclude-computed-then-partial",
        "exclude-computed-nested-omitted",
        "exclude-computed-nested-partial",
        "exclude-nested-omitted",
        "exclude-conditional-required",
        "exclude-conditional-forbidden",
        "exclude-conditional-forbidden-known",
        "exclude-logical-spread",
        "exclude-logical-and-exclude",
        "exclude-logical-or-exclude",
        "exclude-logical-nullish-exclude",
        "exclude-logical-and-known",
        "exclude-identifier-spread",
        "exclude-call-spread",
    ] {
        let root = rules_path("vitest-project-policy", "fixture", "exclude-globs");
        cases.push((name, root.clone(), root.join(format!("{name}.yml"))));
    }
    let root = rules_path(
        "vitest-project-policy",
        "fixture",
        "exclude-globs-suppressed",
    );
    cases.push((
        "exclude-suppressed",
        root.clone(),
        root.join("exclude-suppressed.yml"),
    ));
    let root = rules_path("vitest-project-policy", "fixture", "helper-glob-skip");
    cases.push((
        "helper-glob-config-excluded",
        root.clone(),
        root.join("config-excluded.yml"),
    ));
    cases.push((
        "helper-glob-config-skipped",
        root.clone(),
        root.join("config-skipped.yml"),
    ));
    assert_saved_fixture("vitest_project_policy", &cases);
}
