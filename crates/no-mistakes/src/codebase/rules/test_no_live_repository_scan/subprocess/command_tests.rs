use super::{invocation, matched, AnalyzerCommand, ArgumentValue};
use serde_json::Value;

fn analyzer() -> AnalyzerCommand {
    AnalyzerCommand {
        command: "widget-audit".to_string(),
        args: vec!["check".to_string()],
    }
}

fn known(text: &str) -> ArgumentValue {
    ArgumentValue::Scalar(Value::String(text.to_string()))
}

fn launch(values: Vec<ArgumentValue>) -> String {
    let parsed = invocation(&values).unwrap();
    matched(&parsed, &[analyzer()]).unwrap_or_default()
}

#[test]
fn wrappers_and_basenames_match_the_configured_prefix() {
    let check = ArgumentValue::Array(vec![known("check")]);
    assert_eq!(
        launch(vec![known("widget-audit"), check.clone()]),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![known("./node_modules/.bin/widget-audit"), check]),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![
            known("pnpm"),
            ArgumentValue::Array(vec![known("exec"), known("widget-audit"), known("check")])
        ]),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![
            known("pnpm"),
            ArgumentValue::Array(vec![
                known("dlx"),
                known("--"),
                known("widget-audit"),
                known("check")
            ])
        ]),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![
            known("npx"),
            ArgumentValue::Array(vec![
                known("--yes"),
                known("widget-audit"),
                known("check"),
                known("--json")
            ])
        ]),
        "widget-audit check"
    );
    assert!(launch(vec![
        known("node"),
        ArgumentValue::Array(vec![known("fixtures/print-result.js")])
    ])
    .is_empty());
    assert!(launch(vec![
        known("widget-audit"),
        ArgumentValue::Array(vec![known("list")])
    ])
    .is_empty());
    assert!(launch(vec![
        known("pnpm"),
        ArgumentValue::Array(vec![known("exec"), known("widget-audit")])
    ])
    .is_empty());
    assert_eq!(
        launch(vec![
            known("widget-audit.exe"),
            ArgumentValue::Array(vec![known("check")])
        ]),
        "widget-audit check"
    );
    assert!(launch(vec![
        known("not-widget-audit"),
        ArgumentValue::Array(vec![known("check")])
    ])
    .is_empty());
}
