use super::*;

#[test]
fn root_level_option_type_errors_name_the_options_object() {
    let application = RuleApplicationConfig {
        rule: "boolean-options".to_string(),
        options: serde_yaml::from_str("[not, a, boolean]").unwrap(),
        ..Default::default()
    };

    let error = application
        .try_rule_options::<bool>()
        .expect_err("a sequence cannot deserialize as a boolean options object");

    assert!(
        error.to_string().contains("at options:"),
        "unexpected diagnostic: {error:#}"
    );
}
