use super::{invocation, matched, uses_shell, AnalyzerCommand, ArgumentValue, OptionsSlot, Token};
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

fn argv(items: &[&str]) -> ArgumentValue {
    ArgumentValue::Array(items.iter().copied().map(known).collect())
}

fn shell_flag(value: bool) -> ArgumentValue {
    let mut fields = crate::fx::FxHashMap::default();
    fields.insert(
        "shell".to_string(),
        ArgumentValue::Scalar(Value::Bool(value)),
    );
    ArgumentValue::Object(fields)
}

fn launch(values: Vec<ArgumentValue>, shell: bool) -> String {
    let parsed = invocation(&values, shell).unwrap();
    matched(&parsed, &[analyzer()]).unwrap_or_default()
}

#[test]
fn shell_strings_split_on_whitespace() {
    let command = known("pnpm  exec\twidget-audit check");
    assert_eq!(launch(vec![command.clone()], true), "widget-audit check");
    let parsed = invocation(std::slice::from_ref(&command), true).unwrap();
    assert!(matches!(parsed.options, OptionsSlot::Absent));
    assert!(matches!(parsed.executable, Token::Known(ref text) if text == "pnpm"));

    let with_options = invocation(&[command, shell_flag(true)], true).unwrap();
    assert!(matches!(with_options.options, OptionsSlot::Object(1)));
    assert_eq!(
        matched(&with_options, &[analyzer()]).as_deref(),
        Some("widget-audit check")
    );

    let callback = invocation(
        &[
            known("pnpm exec widget-audit check"),
            ArgumentValue::Unknown,
        ],
        true,
    )
    .unwrap();
    assert!(matches!(callback.options, OptionsSlot::Unknown));
    assert!(matched(&callback, &[analyzer()]).is_some());

    assert!(launch(vec![known("pnpm exec widget-audit check")], false).is_empty());
    assert!(launch(vec![known("pnpm exec other-audit check")], true).is_empty());
    assert!(launch(vec![known("pnpm exec widget-audit list")], true).is_empty());
    assert!(launch(vec![known("")], true).is_empty());
    assert!(launch(vec![ArgumentValue::Unknown], true).is_empty());
    assert_eq!(
        launch(
            vec![
                known("pnpm"),
                argv(&["exec", "widget-audit", "check"]),
                shell_flag(true)
            ],
            false
        ),
        "widget-audit check"
    );
}

#[test]
fn shell_prefixes_are_removed_before_the_executable() {
    assert_eq!(
        launch(vec![known("ITEM=1 widget-audit check")], true),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![known("_ITEM=1 widget-audit check")], true),
        "widget-audit check"
    );
    assert_eq!(
        launch(vec![known("cd widgets && widget-audit check")], true),
        "widget-audit check"
    );
    assert_eq!(
        launch(
            vec![known(
                "cd widgets && ITEM=1 cd items && pnpm exec widget-audit check"
            )],
            true
        ),
        "widget-audit check"
    );
    assert!(launch(vec![known("cd widgets &&")], true).is_empty());
    assert!(launch(vec![known("ITEM=1")], true).is_empty());
    assert!(launch(vec![known("2=1 widget-audit check")], true).is_empty());
    assert!(launch(vec![known("ITEM-COUNT=1 widget-audit check")], true).is_empty());
    assert!(launch(vec![known("cd && widget-audit check")], true).is_empty());
    assert!(launch(vec![known("cd widgets widget-audit check")], true).is_empty());
    assert!(launch(vec![known("cd && && widget-audit check")], true).is_empty());
    assert!(launch(vec![known("cd -widgets && widget-audit check")], true).is_empty());
    assert!(launch(vec![known("cd widgets; widget-audit check")], true).is_empty());
    assert!(launch(vec![known("\"widget-audit\" check")], true).is_empty());
    assert!(launch(
        vec![known("ITEM=\"placeholder widget-audit check extra\"")],
        true
    )
    .is_empty());
}

#[test]
fn shell_mode_follows_the_launcher() {
    let string = vec![known("pnpm exec widget-audit check")];
    assert!(uses_shell("exec", &string));
    assert!(uses_shell("execSync", &string));
    assert!(!uses_shell("execFile", &string));
    assert!(!uses_shell("execFileSync", &string));
    assert!(!uses_shell("fork", &string));
    assert!(!uses_shell("spawn", &string));
    assert!(!uses_shell("spawnSync", &string));

    let flagged = vec![known("pnpm exec widget-audit check"), shell_flag(true)];
    assert!(uses_shell("spawn", &flagged));
    assert!(uses_shell("spawnSync", &flagged));
    assert!(!uses_shell(
        "spawn",
        &[known("pnpm exec widget-audit check"), shell_flag(false)]
    ));

    let mut path_shell = crate::fx::FxHashMap::default();
    path_shell.insert("shell".to_string(), known("/bin/sh"));
    assert!(!uses_shell(
        "spawn",
        &[
            known("pnpm exec widget-audit check"),
            ArgumentValue::Object(path_shell)
        ]
    ));

    let argv_shell = vec![
        known("pnpm"),
        argv(&["exec", "widget-audit", "check"]),
        shell_flag(true),
    ];
    assert!(!uses_shell("exec", &argv_shell));
    assert!(!uses_shell("spawn", &argv_shell));
    assert!(!uses_shell(
        "spawn",
        &[
            known("pnpm exec widget-audit check"),
            ArgumentValue::Unknown
        ]
    ));
}
