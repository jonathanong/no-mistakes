use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn vitest_project_policy_json_matches_snapshot() {
    let mut cases = [
        "pass",
        "declared",
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
    assert_saved_fixture("vitest_project_policy", &cases);
}
