use super::{launches, matched, uses_shell, AnalyzerCommand, ArgumentValue, OptionsSlot, Token};
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
    launches(&values, shell)
        .iter()
        .find_map(|parsed| matched(parsed, &[analyzer()]))
        .unwrap_or_default()
}

fn matched_dir(text: &str) -> Option<String> {
    launches(&[known(text)], true)
        .into_iter()
        .find(|parsed| matched(parsed, &[analyzer()]).is_some())
        .and_then(|parsed| first_shell_dir(&parsed).map(str::to_owned))
}

fn first_shell_dir(parsed: &super::Invocation) -> Option<&str> {
    parsed.shell_dirs.as_ref()?.first()?.as_deref()
}

fn matched_text(text: &str) -> String {
    launch(vec![known(text)], true)
}

fn assert_matched(text: &str) {
    assert_eq!(matched_text(text), "widget-audit check");
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
    assert!(launch(vec![known("cd widgets widget-audit check")], true).is_empty());
    assert!(launch(
        vec![known("ITEM=\"placeholder widget-audit check extra\"")],
        true
    )
    .is_empty());
}

#[test]
fn quoted_arguments_match_on_static_text() {
    assert_matched("widget-audit check --label 'nightly'");
    assert_matched("widget-audit check --label \"nightly\"");
    let parsed = invocation(&[known("widget-audit check --label 'nightly'")], true).unwrap();
    assert!(matches!(parsed.argv.last(), Some(Token::Known(text)) if text == "nightly"));
    let quoted_dir = invocation(
        &[known(
            "cd \"widgets\" && widget-audit check --label 'nightly'",
        )],
        true,
    )
    .unwrap();
    assert_eq!(
        matched(&quoted_dir, &[analyzer()]).as_deref(),
        Some("widget-audit check")
    );
    assert_eq!(first_shell_dir(&quoted_dir), Some("widgets"));
    let literal = invocation(&[known("widget-audit check --label '$item'")], true).unwrap();
    assert_eq!(
        matched(&literal, &[analyzer()]).as_deref(),
        Some("widget-audit check")
    );
    assert!(matches!(literal.argv.last(), Some(Token::Known(text)) if text == "$item"));
    assert_matched("widget-audit check --label '`item`'");
    assert_matched("ITEM='$item' widget-audit check");
    assert!(matched_text("'ITEM=$item' widget-audit check").is_empty());
    let quoted_operator = invocation(&[known("cd \"&&\" && widget-audit check")], true).unwrap();
    assert_eq!(
        matched(&quoted_operator, &[analyzer()]).as_deref(),
        Some("widget-audit check")
    );
    assert_eq!(first_shell_dir(&quoted_operator), Some("&&"));
    let quoted_cd = invocation(&[known("\"cd\" widgets && widget-audit check")], true).unwrap();
    assert_eq!(
        matched(&quoted_cd, &[analyzer()]).as_deref(),
        Some("widget-audit check")
    );
    assert_eq!(first_shell_dir(&quoted_cd), Some("widgets"));
    let partial_cd = invocation(&[known("c''d widgets && widget-audit check")], true).unwrap();
    assert_eq!(
        matched(&partial_cd, &[analyzer()]).as_deref(),
        Some("widget-audit check")
    );
    assert_eq!(first_shell_dir(&partial_cd), Some("widgets"));
    assert!(matched_text("\"cd\" widgets \"&&\" widget-audit check").is_empty());
    assert!(matched_text("widget-audit check --label 'nightly").is_empty());
    assert!(matched_text("widget-audit check --label 'ni\\ghtly'").is_empty());
    assert!(matched_text("widget-audit check --label \"$item\"").is_empty());
    assert!(matched_text("widget-audit check --label \"`item`\"").is_empty());
    assert!(matched_text("widget-audit check $item --label 'nightly'").is_empty());
    assert!(matched_text("ITEM=\"$item\" widget-audit check").is_empty());
    assert!(matched_text("cd widgets \"&&\" widget-audit check").is_empty());
}

#[test]
fn operators_and_quoted_executables_expose_each_simple_command() {
    assert_matched("true && widget-audit check");
    assert_matched("true&&widget-audit check");
    assert_matched("true; widget-audit check");
    assert_matched("true||widget-audit check");
    assert_matched("\"widget-audit\" check");
    assert_matched("widget-au''dit check");
    assert_matched("cd && widget-audit check");
    assert_matched("cd && && widget-audit check");
    assert_matched("cd -widgets && widget-audit check");
    assert_matched("cd widgets; widget-audit check");
    assert_matched("cd widgets || widget-audit check");
    assert_eq!(
        matched_dir("cd widgets; widget-audit check").as_deref(),
        Some("widgets")
    );
    assert_eq!(
        matched_dir("cd \"widgets\" && widget-audit check").as_deref(),
        Some("widgets")
    );
    assert_eq!(
        matched_dir("cd widgets || widget-audit check").as_deref(),
        None
    );
    assert_eq!(matched_dir("true && widget-audit check").as_deref(), None);
    assert!(matched_text("cd fixtures/$DIR && widget-audit check").is_empty());
    assert!(matched_text("cd fixtures/`item` && widget-audit check").is_empty());
    assert!(matched_text("widget-audit check `date`").is_empty());
    assert!(matched_text("ITEM=$item widget-audit check").is_empty());
    assert!(matched_text("widget-audit check \\--json").is_empty());
    assert!(matched_text("cd widgets | widget-audit check").is_empty());
    assert!(matched_text("cd widgets & widget-audit check").is_empty());
    assert_eq!(
        matched_dir("cd 'fixtures/$item' && widget-audit check").as_deref(),
        Some("fixtures/$item")
    );
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
