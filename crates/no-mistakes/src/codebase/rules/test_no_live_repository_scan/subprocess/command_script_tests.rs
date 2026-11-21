use super::{launches, matched, AnalyzerCommand, ArgumentValue};
use serde_json::Value;

fn invocation(values: &[ArgumentValue], shell: bool) -> Option<super::Invocation> {
    launches(values, shell).into_iter().next()
}

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
    let parsed = invocation(&values, false).unwrap();
    matched(&parsed, &[analyzer()]).unwrap_or_default()
}

#[test]
fn script_extensions_match_the_configured_program() {
    assert_eq!(
        launch(vec![
            known("./widget-audit.js"),
            ArgumentValue::Array(vec![known("check")])
        ]),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![
            known("./widgets/widget-audit.mjs"),
            ArgumentValue::Array(vec![known("check")])
        ]),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![
            known("widget-audit.cjs"),
            ArgumentValue::Array(vec![known("check")])
        ]),
        "widget-audit check"
    );
    assert!(launch(vec![
        known("./other-audit.js"),
        ArgumentValue::Array(vec![known("check")])
    ])
    .is_empty());
}
