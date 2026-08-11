use super::*;

#[test]
fn fact_runner_reports_invalid_rule_include_globs() {
    let root = fixture();
    let mut config = config();
    config.rules[0].include = vec!["[".to_string()];

    let error = check_with_facts(&root, &config, &CheckFactMap::default()).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("rule `nextjs-no-caching` include contains invalid glob"),
        "{error:#}"
    );
}
