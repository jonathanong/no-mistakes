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
        ("fail.yml", "advise"),
        ("upper-fail.yml", "advise"),
        ("canonical-report.yml", "advise"),
        ("canonical-advise.yml", "advise"),
        ("canonical-ignore.yml", "ignore"),
        ("canonical-over-fail.yml", "ignore"),
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
        let expected_policy = if expected == "advise" {
            crate::config::v2::Unanalyzable::Advise
        } else {
            crate::config::v2::Unanalyzable::Ignore
        };
        assert_eq!(
            config
                .requested_rule_analysis(&config.rules[0])
                .unwrap()
                .unanalyzable,
            expected_policy,
            "{name}"
        );
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .unanalyzable,
            expected_policy,
            "{name}"
        );
        assert_eq!(
            crate::codebase::postgres::has_sql_advice(&config),
            expected == "advise",
            "{name}"
        );
    }
}

#[test]
fn grouping_unanalyzable_options_accept_default_and_normalized_ignore() {
    for (name, advice) in [
        ("default.yml", true),
        ("empty.yml", true),
        ("upper-ignore.yml", false),
    ] {
        let config = load(name);
        compile_rule(&config);
        assert_eq!(
            crate::codebase::postgres::has_sql_advice(&config),
            advice,
            "{name}"
        );
        if name == "default.yml" {
            assert!(config.rules[0].options.get("unanalyzableSql").is_none());
        }
    }
}

#[test]
fn invalid_sql_modes_are_rejected_and_legacy_fail_compiles() {
    let config = load("invalid.yml");
    let opts: Options = config.rules[0].try_rule_options().unwrap();
    let error = compile::options(&opts).err().unwrap().to_string();
    assert!(error.contains("unanalyzableSql must be"), "{error}");
    let mut opts = opts;
    opts.unanalyzable_sql = "fail".into();
    compile::options(&opts).unwrap();
}

#[test]
fn canonical_policy_still_rejects_an_invalid_explicit_alias() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/postgres-grouping-policy");
    let error = crate::config::v2::load_v2_config(&root, Some(Path::new("invalid-canonical.yml")))
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid options.unanalyzableSql"), "{error}");
}

#[test]
fn canonical_policy_rejects_the_legacy_fail_vocabulary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/postgres-grouping-policy");
    let error =
        crate::config::v2::load_v2_config(&root, Some(Path::new("canonical-invalid-fail.yml")))
            .unwrap_err()
            .to_string();
    assert!(error.contains("invalid options.unanalyzable"), "{error}");
}
