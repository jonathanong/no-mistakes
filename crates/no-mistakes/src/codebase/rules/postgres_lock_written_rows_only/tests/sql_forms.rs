use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-written-rows-only/fixture/sql-lock-forms")
}

fn config(policy: &str) -> NoMistakesConfig {
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/checks-fail.yml"
    )))
    .unwrap();
    config.rules[0].options["unanalyzableSql"] = serde_yaml::to_value(policy).unwrap();
    config
}

fn findings(policy: &str) -> Vec<(usize, String)> {
    let mut findings: Vec<_> = check(&fixture(), &config(policy))
        .unwrap()
        .into_iter()
        .map(|finding| {
            assert_eq!(finding.rule, RULE_ID);
            assert_eq!(finding.file, "src/query.ts");
            (finding.line, finding.target.unwrap())
        })
        .collect();
    findings.sort();
    findings
}

#[test]
fn alias_derived_join_and_qualified_relation_proofs_keep_their_boundaries() {
    let findings = findings("fail");
    let unresolved: Vec<_> = findings
        .iter()
        .filter(|(_, target)| target == "unanalyzable")
        .map(|(line, _)| *line)
        .collect();
    assert_eq!(unresolved, [9, 14, 33]);
    assert_eq!(
        findings
            .iter()
            .filter(|(line, target)| *line == 18 && target == CHECKS[0])
            .count(),
        1,
        "a resolved joined OF alias with no matching write is an unwritten relation"
    );
    assert!(
        !findings.iter().any(|(line, _)| matches!(line, 4 | 38)),
        "matching OF aliases and exact qualified relations must remain compliant: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|(line, target)| { target != CHECKS[0] || *line == 18 }),
        "unsupported relation proofs must not become unwritten-row violations: {findings:?}"
    );
}

#[test]
fn cast_unary_arithmetic_and_qualified_advisory_constants_are_global_locks() {
    let findings = findings("ignore");
    assert_eq!(
        findings,
        [22, 23, 24, 25, 26, 27]
            .map(|line| (line, CHECKS[2].to_owned()))
            .to_vec()
    );
    assert!(
        !findings.iter().any(|(line, _)| *line == 28),
        "a hash derived from a row key remains compliant"
    );
}
