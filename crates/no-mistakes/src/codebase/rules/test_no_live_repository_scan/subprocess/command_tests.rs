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
    let parsed = invocation(&values, false).unwrap();
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
    assert_eq!(
        launch(vec![
            known("npx"),
            ArgumentValue::Array(vec![
                known("--yes"),
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
                known("widget-audit"),
                known("check"),
                known("--"),
                known("--json")
            ])
        ]),
        "widget-audit check"
    );
    assert_eq!(
        launch_as(
            "widget-audit.exe",
            vec![
                known("./node_modules/.bin/widget-audit.exe"),
                ArgumentValue::Array(vec![known("check")])
            ]
        ),
        "widget-audit.exe check"
    );
}

#[test]
fn incomplete_argv_does_not_match_the_configured_prefix() {
    let executable = invocation(&[known("widget-audit")], false).unwrap();
    assert!(matches!(executable.options, super::OptionsSlot::Absent));
    assert!(executable.argv.is_empty());
    assert!(matched(&executable, &[analyzer()]).is_none());

    let options_slot = invocation(
        &[
            known("widget-audit"),
            ArgumentValue::Object(crate::fx::FxHashMap::default()),
        ],
        false,
    )
    .unwrap();
    assert!(matches!(
        options_slot.options,
        super::OptionsSlot::Object(1)
    ));
    assert!(options_slot.argv.is_empty());
    assert!(matched(&options_slot, &[analyzer()]).is_none());

    let dynamic_argv = invocation(&[known("widget-audit"), ArgumentValue::Unknown], false).unwrap();
    assert!(matches!(dynamic_argv.options, super::OptionsSlot::Unknown));
    assert!(matches!(
        dynamic_argv.argv.as_slice(),
        [super::Token::Unknown]
    ));

    let dynamic_executable = invocation(&[ArgumentValue::Unknown], false).unwrap();
    assert!(matches!(
        dynamic_executable.executable,
        super::Token::Unknown
    ));
    assert!(matched(&dynamic_executable, &[analyzer()]).is_none());

    assert!(launch(vec![
        known("pnpm"),
        ArgumentValue::Array(vec![
            ArgumentValue::Unknown,
            known("widget-audit"),
            known("check")
        ])
    ])
    .is_empty());
    assert!(launch(vec![
        known("pnpm"),
        ArgumentValue::Array(vec![known("run"), known("widget-audit"), known("check")])
    ])
    .is_empty());
    assert!(launch(vec![known("npx"), ArgumentValue::Array(vec![])]).is_empty());
    assert!(launch(vec![
        known("pnpm"),
        ArgumentValue::Array(vec![known("exec"), ArgumentValue::Unknown, known("check")])
    ])
    .is_empty());
}

fn launch_as(command: &str, values: Vec<ArgumentValue>) -> String {
    let parsed = invocation(&values, false).unwrap();
    matched(
        &parsed,
        &[AnalyzerCommand {
            command: command.to_string(),
            args: vec!["check".to_string()],
        }],
    )
    .unwrap_or_default()
}
