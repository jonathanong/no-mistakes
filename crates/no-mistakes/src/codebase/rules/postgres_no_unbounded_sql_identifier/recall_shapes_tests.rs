use super::findings;

#[test]
fn remaining_identifier_shapes_report_exact_sources() {
    for (scenario, line, producer) in [
        ("fail-conditional-env", 3, "process.env.SCHEMA"),
        ("fail-template-validator", 6, "process.env.SCHEMA"),
        ("fail-concat-env", 2, "process.env.SCHEMA"),
        ("fail-alias-chain", 4, "b"),
        ("fail-env-object-alias", 3, "process.env.SCHEMA"),
        ("fail-process-namespace", 3, "process.env.SCHEMA"),
        ("fail-destructured-rows", 3, "rows"),
        ("fail-destructured-row", 4, "schema"),
        ("fail-array-destructured-row", 5, "id"),
        ("fail-array-destructured-row-elision", 4, "schema"),
        ("fail-comma-env", 3, "process.env.SCHEMA"),
        ("fail-iterated-rows", 4, "row"),
        ("fail-iterated-row-alias", 4, "row"),
    ] {
        let found = findings(scenario);
        assert_eq!(found.len(), 1, "{scenario}: {found:#?}");
        assert_eq!(found[0].file, "src/query.ts", "{scenario}");
        assert_eq!(found[0].line, line, "{scenario}");
        assert_eq!(found[0].target.as_deref(), Some(producer), "{scenario}");
        assert!(found[0].message.contains("finite-domain validator"));
    }
}

#[test]
fn remaining_identifier_shapes_preserve_finite_bind_and_unknown_controls() {
    for scenario in [
        "pass-conditional-literals",
        "pass-conditional-validator-local",
        "pass-conditional-validator-nested",
        "pass-conditional-validator-imported",
        "pass-new-bind-value",
        "pass-projected-http-rows",
        "pass-array-destructured-http-row",
        "pass-array-destructured-mutable-row",
        "pass-array-destructured-mutable-result",
        "pass-empty-and-computed-patterns",
        "pass-comma-discarded-env",
        "pass-alias-cycle",
        "pass-alias-shadow",
        "pass-finite-alias",
        "pass-process-namespace-shadow",
        "pass-env-alias-process-parameter",
    ] {
        assert!(
            findings(scenario).is_empty(),
            "{scenario}: {:#?}",
            findings(scenario)
        );
    }
}
