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
fn package_manager_flags_and_nested_wrappers_unwrap() {
    assert!(matches(
        "pnpm",
        &["-C", "dir", "exec", "widget-audit", "check"]
    ));
    assert!(matches(
        "pnpm",
        &["--filter", "widgets", "exec", "widget-audit", "check"]
    ));
    assert!(matches(
        "pnpm",
        &["--filter=widgets", "exec", "widget-audit", "check"]
    ));
    assert!(matches(
        "pnpm",
        &[
            "-Cdir",
            "--dir",
            "dir",
            "-r",
            "exec",
            "widget-audit",
            "check"
        ]
    ));
    assert!(matches(
        "pnpm",
        &["-F", "widgets", "dlx", "--", "widget-audit", "check"]
    ));
    assert!(!matches(
        "pnpm",
        &["-C", "dir", "run", "widget-audit", "check"]
    ));
    assert!(!matches("pnpm", &["-C"]));
    assert!(!matches("pnpm", &["--filter", "widget-audit", "check"]));

    assert!(matches("npx", &["--no-install", "widget-audit", "check"]));
    assert!(matches(
        "npx",
        &["-p", "widget-tools", "widget-audit", "check"]
    ));
    assert!(matches(
        "npx",
        &["--package=widget-tools", "widget-audit", "check"]
    ));
    assert!(!matches("npx", &["-p", "widget-audit", "check"]));
    assert!(!matches("npx", &["-p"]));

    assert!(matches("npm", &["exec", "--", "widget-audit", "check"]));
    assert!(!matches("npm", &["run", "widget-audit", "check"]));

    assert!(matches("yarn", &["widget-audit", "check"]));
    assert!(matches("yarn", &["dlx", "widget-audit", "check"]));
    assert!(matches("yarn.cmd", &["dlx", "--", "widget-audit", "check"]));
    assert!(matches("yarn", &["run", "widget-audit", "check"]));
    assert!(matches(
        "yarn",
        &["--cwd", "widgets", "widget-audit", "check"]
    ));
    assert!(matches(
        "yarn",
        &["--cwd=widgets", "run", "--", "widget-audit", "check"]
    ));
    assert!(!matches("yarn", &["--cwd", "widget-audit", "check"]));
    assert!(!matches("yarn", &["--cwd"]));
    assert!(matches("bunx", &["--yes", "widget-audit", "check"]));

    assert!(matches(
        "npx",
        &["pnpm", "-C", "dir", "exec", "widget-audit", "check"]
    ));
    assert!(matches(
        "npx",
        &[
            "bunx",
            "yarn",
            "dlx",
            "pnpm",
            "exec",
            "widget-audit",
            "check"
        ]
    ));
    assert!(!matches(
        "npx",
        &[
            "npx",
            "bunx",
            "yarn",
            "dlx",
            "pnpm",
            "exec",
            "widget-audit",
            "check"
        ]
    ));
}

#[test]
fn an_unknown_flag_value_does_not_unwrap() {
    let values = vec![
        known("pnpm"),
        ArgumentValue::Array(vec![
            known("-C"),
            ArgumentValue::Unknown,
            known("exec"),
            known("widget-audit"),
            known("check"),
        ]),
    ];
    let parsed = invocation(&values, false).unwrap();
    assert!(matched(&parsed, &[analyzer()]).is_none());
}
