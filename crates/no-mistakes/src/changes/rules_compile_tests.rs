use crate::changes::config::{ChangeCategory, ChangeRule, ChangesConfig, IdRule};

fn category(category: ChangeCategory) -> ChangesConfig {
    let mut config = ChangesConfig::default();
    config.categories.insert("api".to_string(), category);
    config
}

fn compile_error(config: &ChangesConfig) -> String {
    match super::Rules::compile(config) {
        Ok(_) => panic!("expected changes compilation to fail"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn compile_rejects_invalid_exclude_and_path_globs() {
    let exclude = category(ChangeCategory {
        paths: vec!["src/**".to_string()],
        exclude: vec!["[".to_string()],
        ..ChangeCategory::default()
    });
    let error = compile_error(&exclude);
    assert!(error.contains("exclude"), "{error}");

    let paths = category(ChangeCategory {
        paths: vec!["[".to_string()],
        ..ChangeCategory::default()
    });
    let error = compile_error(&paths);
    assert!(error.contains("paths"), "{error}");
}

#[test]
fn compile_rejects_a_from_path_without_one_placeholder() {
    let config = category(ChangeCategory {
        paths: vec!["src/**".to_string()],
        id: Some(IdRule {
            from_path: Some("no-placeholder".to_string()),
            ..IdRule::default()
        }),
        ..ChangeCategory::default()
    });
    let error = compile_error(&config);
    assert!(error.contains("ID rule"), "{error}");
}

#[test]
fn compile_rejects_a_rule_without_paths() {
    let config = category(ChangeCategory {
        rules: vec![ChangeRule {
            id: Some(IdRule {
                from_path: Some("{id}".to_string()),
                ..IdRule::default()
            }),
            ..ChangeRule::default()
        }],
        ..ChangeCategory::default()
    });
    let error = compile_error(&config);
    assert!(error.contains("requires paths"), "{error}");
}
