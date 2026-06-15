use crate::config::v2::schema::{ImpactConfig, NoMistakesConfig};

#[test]
fn impact_defaults_to_empty() {
    let cfg = NoMistakesConfig::default();
    assert_eq!(cfg.tests.impact, ImpactConfig::default());
    assert!(cfg.tests.impact.always_include_tests.is_empty());
    assert!(cfg.tests.impact.registries.is_empty());
}

#[test]
fn impact_parses_camel_case_keys() {
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
tests:
  impact:
    alwaysIncludeTests:
      - "**/*.mock.test.*"
    registries:
      - "**/auth-gated-code-splitting.mts"
      - "**/*-registry.mts"
"#,
    )
    .unwrap();

    assert_eq!(
        cfg.tests.impact.always_include_tests,
        vec!["**/*.mock.test.*".to_string()]
    );
    assert_eq!(
        cfg.tests.impact.registries,
        vec![
            "**/auth-gated-code-splitting.mts".to_string(),
            "**/*-registry.mts".to_string()
        ]
    );
}

#[test]
fn impact_round_trips_through_serialization() {
    let config = ImpactConfig {
        always_include_tests: vec!["**/*.mock.test.*".to_string()],
        registries: vec!["**/*-registry.mts".to_string()],
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: ImpactConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(config, parsed);
}
