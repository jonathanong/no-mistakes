use super::findings;

#[test]
fn remaining_registry_shapes_report_the_exact_producer_and_writer() {
    for scenario in [
        "fail-literal-computed",
        "fail-dynamic-key",
        "fail-static-require",
        "fail-many-const-requires",
        "fail-require-namespace",
        "fail-namespace-reexport",
        "fail-namespace-named-reexport",
    ] {
        let found = findings(scenario);
        assert_eq!(found.len(), 1, "{scenario}: {found:#?}");
        assert_eq!(found[0].file, "src/query.ts", "{scenario}");
        assert_eq!(found[0].line, 3, "{scenario}");
        assert_eq!(
            found[0].target.as_deref(),
            Some("src/tables.ts"),
            "{scenario}"
        );
        assert!(
            found[0].message.contains("src/mutate.ts"),
            "{scenario}: {found:#?}"
        );
    }
}

#[test]
fn unknown_require_and_unrelated_dynamic_writes_stay_silent() {
    for scenario in ["pass-dynamic-require", "pass-dynamic-unrelated"] {
        assert!(
            findings(scenario).is_empty(),
            "{scenario}: {:#?}",
            findings(scenario)
        );
    }
}
