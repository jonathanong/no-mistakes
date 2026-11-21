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

fn matches(program: &str, args: &[&str]) -> bool {
    let values = vec![
        known(program),
        ArgumentValue::Array(args.iter().copied().map(known).collect()),
    ];
    let parsed = invocation(&values, false).unwrap();
    matched(&parsed, &[analyzer()]).as_deref() == Some("widget-audit check")
}

#[test]
fn reporter_values_unwrap_and_other_flags_do_not() {
    assert!(matches(
        "pnpm",
        &["--reporter", "silent", "exec", "widget-audit", "check"]
    ));
    assert!(matches(
        "pnpm",
        &["--reporter=silent", "-r", "exec", "widget-audit", "check"]
    ));
    assert!(!matches(
        "pnpm",
        &["--ignore-scripts", "exec", "widget-audit", "check"]
    ));
    assert!(!matches("pnpm", &["-s", "exec", "widget-audit", "check"]));
    assert!(!matches(
        "pnpm",
        &[
            "--offline",
            "--reporter",
            "silent",
            "exec",
            "widget-audit",
            "check"
        ]
    ));
    assert!(!matches("pnpm", &["--reporter"]));
    assert!(!matches(
        "npm",
        &["--reporter", "exec", "widget-audit", "check"]
    ));
    assert!(matches(
        "pnpm",
        &["--recursive", "exec", "widget-audit", "check"]
    ));
    assert!(matches(
        "pnpm",
        &["-Fwidgets", "dlx", "widget-audit", "check"]
    ));
    assert!(!matches("yarn", &[]));
    assert!(!matches("yarn", &["--cwd=", "widget-audit", "check"]));

    let reporter = vec![
        known("pnpm"),
        ArgumentValue::Array(vec![
            known("--reporter"),
            ArgumentValue::Unknown,
            known("exec"),
            known("widget-audit"),
            known("check"),
        ]),
    ];
    let parsed = invocation(&reporter, false).unwrap();
    assert!(matched(&parsed, &[analyzer()]).is_none());

    let values = vec![
        known("yarn"),
        ArgumentValue::Array(vec![
            known("--cwd"),
            ArgumentValue::Unknown,
            known("widget-audit"),
            known("check"),
        ]),
    ];
    let parsed = invocation(&values, false).unwrap();
    assert!(matched(&parsed, &[analyzer()]).is_none());
}
