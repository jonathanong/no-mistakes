use super::*;

fn findings(case: &str, policy: &str) -> Vec<(usize, String)> {
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/checks-fail.yml"
    )))
    .unwrap();
    config.rules[0].options["unanalyzableSql"] = serde_yaml::to_value(policy).unwrap();
    let mut findings: Vec<_> = check(&fixture(case), &config)
        .unwrap()
        .into_iter()
        .map(|finding| {
            assert_eq!(finding.rule, RULE_ID);
            (finding.line, finding.target.unwrap())
        })
        .collect();
    findings.sort();
    findings
}

#[test]
fn destructured_parameters_and_constants_preserve_each_binding_identity() {
    assert_eq!(
        findings("destructured-keys", "fail"),
        [4, 14, 25, 37]
            .map(|line| (line, CHECKS[0].to_owned()))
            .to_vec()
    );
}

#[test]
fn return_or_throw_before_a_matching_write_abstains_without_tainting_earlier_writes() {
    let controls = findings("early-exit", "fail");
    assert_eq!(
        controls,
        vec![
            (4, "unanalyzable".to_owned()),
            (10, "unanalyzable".to_owned())
        ]
    );
    assert!(
        controls
            .iter()
            .all(|(line, _)| !matches!(line, 29 | 30 | 34 | 35)),
        "nested returns and returned query arguments preserve unconditional writes: {controls:?}"
    );
    for policy in ["advise", "ignore"] {
        assert!(findings("early-exit", policy).is_empty(), "{policy}");
    }
}
