use super::*;

#[test]
fn helper_sql_policies_preserve_known_findings_origins_and_residual_policy() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-helper-recovery");
    let caller_source = std::fs::read_to_string(root.join("src/policy-consumers.mts")).unwrap();
    let helper_source = std::fs::read_to_string(root.join("src/helpers.mts")).unwrap();
    let residual = marked(&caller_source, "OFFSET_WITH_RESIDUAL")[0];
    let uuid = marked(&caller_source, "TRANSFORMED_UUID")[0];
    let uuid_residual = marked(&caller_source, "UUID_WITH_RESIDUAL")[0];
    let identifier = marked(&caller_source, "UNBOUNDED_IDENTIFIER")[0];
    let awaited = marked(&caller_source, "ASYNC_FRAGMENT")[0];
    let unawaited = marked(&caller_source, "UNAWAITED_PROMISE")[0];
    let helper_offset = marked(&helper_source, "OFFSET_ORIGIN")[0];
    let rules = [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
        "postgres-no-transformed-uuid-predicates",
    ];

    for (mode, config) in [
        ("advise", ".no-mistakes-low.yml"),
        ("report", ".no-mistakes-low-report.yml"),
        ("ignore", ".no-mistakes-low-ignore.yml"),
    ] {
        let result = super::super::run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            false,
        )
        .unwrap();
        assert!(result.warnings.is_empty(), "{mode}: {:?}", result.warnings);

        assert!(
            result.rules.iter().any(|row| {
                row.rule == "postgres-no-offset"
                    && row.file == "src/helpers.mts"
                    && row.line == helper_offset
            }),
            "{mode}: helper OFFSET origin missing: {:?}",
            result.rules
        );
        assert!(
            result.rules.iter().any(|row| {
                row.rule == "postgres-no-unbounded-sql-identifier"
                    && row.file == "src/policy-consumers.mts"
                    && row.line == identifier
            }),
            "{mode}: concrete identifier violation missing: {:?}",
            result.rules
        );
        assert!(
            result.rules.iter().any(|row| {
                row.rule == "postgres-no-transformed-uuid-predicates"
                    && row.file == "src/policy-consumers.mts"
                    && row.line == uuid
            }),
            "{mode}: transformed UUID violation missing: {:?}; advisories: {:?}",
            result.rules,
            result.advisories
        );
        assert!(
            result.rules.iter().any(|row| {
                row.rule == "postgres-no-transformed-uuid-predicates"
                    && row.file == "src/policy-consumers.mts"
                    && row.line == uuid_residual
            }),
            "{mode}: known UUID violation with residual missing: {:?}",
            result.rules
        );
        assert_eq!(
            result.advisories.iter().any(|row| {
                row.rule == "postgres-no-transformed-uuid-predicates"
                    && row.file == "src/policy-consumers.mts"
                    && row.line == uuid_residual
            }),
            mode == "advise",
            "{mode}: residual advice must coexist with the UUID violation"
        );

        for rule in rules {
            let residual_findings = result.rules.iter().any(|row| {
                row.rule == rule && row.file == "src/policy-consumers.mts" && row.line == residual
            });
            let residual_advice = result.advisories.iter().any(|row| {
                row.rule == rule && row.file == "src/policy-consumers.mts" && row.line == residual
            });
            assert_eq!(
                residual_findings,
                mode == "report",
                "{mode} {rule} residual finding: {:?}",
                result.rules
            );
            assert_eq!(
                residual_advice,
                mode == "advise",
                "{mode} {rule} residual advice: {:?}",
                result.advisories
            );
            let promise_findings = result.rules.iter().any(|row| {
                row.rule == rule && row.file == "src/policy-consumers.mts" && row.line == unawaited
            });
            let promise_advice = result.advisories.iter().any(|row| {
                row.rule == rule && row.file == "src/policy-consumers.mts" && row.line == unawaited
            });
            assert_eq!(
                promise_findings,
                mode == "report",
                "{mode} {rule} unawaited SQL"
            );
            assert_eq!(
                promise_advice,
                mode == "advise",
                "{mode} {rule} unawaited SQL"
            );
            assert!(
                !result.rules.iter().any(|row| {
                    row.rule == rule
                        && row.file == "src/policy-consumers.mts"
                        && row.line == awaited
                }),
                "awaited modeled SQL should be complete for {rule}"
            );
            assert!(
                !result.advisories.iter().any(|row| {
                    row.rule == rule
                        && row.file == "src/policy-consumers.mts"
                        && row.line == awaited
                }),
                "awaited modeled SQL should not advise for {rule}"
            );
        }
    }
}
