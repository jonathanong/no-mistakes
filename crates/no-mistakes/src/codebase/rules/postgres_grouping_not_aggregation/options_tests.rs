use super::*;

fn load(name: &str) -> NoMistakesConfig {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/postgres-grouping-policy");
    crate::config::v2::load_v2_config(&root, Some(Path::new(name))).unwrap()
}

fn compile_rule(config: &NoMistakesConfig) {
    let opts: Options = config.rules[0].try_rule_options().unwrap();
    compile::options(&opts).unwrap();
}

#[test]
fn shared_and_project_policies_compile_without_changing_advisory_only_behavior() {
    for (name, expected) in [
        ("shared-report.yml", "advise"),
        ("shared-advise.yml", "advise"),
        ("shared-ignore.yml", "ignore"),
        ("project-fail.yml", "advise"),
        ("project-advise.yml", "advise"),
        ("project-ignore.yml", "ignore"),
    ] {
        let config = load(name);
        assert_eq!(
            config.rules[0].options["unanalyzableSql"], expected,
            "{name}"
        );
        compile_rule(&config);
        assert_eq!(
            crate::codebase::postgres::has_sql_advice(&config),
            expected == "advise",
            "{name}"
        );
    }
}

#[test]
fn grouping_unanalyzable_options_accept_default_and_normalized_ignore() {
    for name in ["empty.yml", "upper-ignore.yml"] {
        let config = load(name);
        compile_rule(&config);
    }
}

#[test]
fn invalid_and_failing_sql_modes_have_an_advisory_only_diagnostic() {
    let config = load("invalid.yml");
    let opts: Options = config.rules[0].try_rule_options().unwrap();
    let error = compile::options(&opts).err().unwrap().to_string();
    assert!(error.contains("must be `ignore` or `advise`"), "{error}");
    let mut opts = opts;
    opts.unanalyzable_sql = "fail".into();
    let error = compile::options(&opts).err().unwrap().to_string();
    assert!(error.contains("advisory only"), "{error}");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/postgres-grouping-policy");
    let error = crate::config::v2::load_v2_config(&root, Some(Path::new("fail.yml")))
        .unwrap_err()
        .to_string();
    assert!(error.contains("must be `ignore` or `advise`"), "{error}");
}
