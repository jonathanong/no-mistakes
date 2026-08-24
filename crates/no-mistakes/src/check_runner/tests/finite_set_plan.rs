use no_mistakes::codebase::check_facts::CheckFactPlan;
use no_mistakes::config::v2::NoMistakesConfig;

#[test]
fn prepare_surfaces_invalid_finite_set_options() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
rules:
  - rule: finite-set-consistency
    scope: repository
    options:
      sets: false
"#,
    )
    .unwrap();
    let mut plan = CheckFactPlan::default();

    let error = crate::check_runner::finite_set_plan::prepare(
        std::path::Path::new("/repo"),
        &config,
        &mut plan,
        false,
        false,
    )
    .err()
    .expect("invalid finite-set options must fail fact planning");

    assert!(error.to_string().contains("options.sets"), "{error:#}");
}
