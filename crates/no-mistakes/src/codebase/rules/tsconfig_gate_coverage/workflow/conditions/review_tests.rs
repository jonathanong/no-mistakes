use super::{evaluation::static_bool, InputState, StaticBool, StaticValue};
use serde_yaml::Value;

#[test]
fn case_functions_resolve_the_selected_static_branch() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("case(true, false, true)", StaticBool::False),
        ("case(false, true, true, false, true)", StaticBool::False),
        (
            "case(false, true, true, 'release', false)",
            StaticBool::True,
        ),
        ("case(true, false, github.ref)", StaticBool::False),
        (
            "case(false, 'ignored', 'release') == 'release'",
            StaticBool::True,
        ),
        ("case(github.ref, false, true)", StaticBool::Unknown),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn over_budget_logical_conditions_never_enter_the_recursive_evaluator() {
    let at_limit = std::iter::repeat_n("true", 256)
        .collect::<Vec<_>>()
        .join(" && ");
    let over_limit = format!("{at_limit} && true");

    assert_eq!(
        static_bool(Some(&Value::String(at_limit)), &InputState::new()),
        StaticBool::True
    );
    assert_eq!(
        static_bool(Some(&Value::String(over_limit)), &InputState::new()),
        StaticBool::Unknown
    );
}

#[test]
fn bracketed_github_event_name_access_matches_dot_access() {
    let inputs = InputState::from([
        (
            "\0github.event_name".into(),
            StaticValue::String("push".into()),
        ),
        (
            "\0github.event.action".into(),
            StaticValue::String("synchronize".into()),
        ),
    ]);
    for (expression, expected) in [
        ("github.event_name == 'push'", StaticBool::True),
        ("github['event_name'] == 'push'", StaticBool::True),
        (
            "GITHUB [ 'EVENT_NAME' ] == 'pull_request'",
            StaticBool::False,
        ),
        ("github[\"event_name\"] == 'push'", StaticBool::Unknown),
        ("github['event_name'].nested == 'push'", StaticBool::Unknown),
        ("github.event.action == 'synchronize'", StaticBool::True),
        ("github.event['action'] == 'synchronize'", StaticBool::True),
        (
            "GITHUB [ 'EVENT' ] [ 'ACTION' ] == 'closed'",
            StaticBool::False,
        ),
        (
            "github['event']['action'].nested == 'synchronize'",
            StaticBool::Unknown,
        ),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn direct_event_action_truthiness_uses_the_event_activation_state() {
    let push_inputs = InputState::from([(
        "\0github.event.action".into(),
        StaticValue::String(String::new()),
    )]);
    let pull_request_inputs = InputState::from([(
        "\0github.event.action".into(),
        StaticValue::String("synchronize".into()),
    )]);

    for (expression, expected_on_push, expected_on_pull_request) in [
        ("github.event.action", StaticBool::False, StaticBool::True),
        (
            "github.event['action']",
            StaticBool::False,
            StaticBool::True,
        ),
        (
            "github['event']['action']",
            StaticBool::False,
            StaticBool::True,
        ),
        ("!github.event.action", StaticBool::True, StaticBool::False),
        (
            "!github.event['action']",
            StaticBool::True,
            StaticBool::False,
        ),
        (
            "!github['event']['action']",
            StaticBool::True,
            StaticBool::False,
        ),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &push_inputs),
            expected_on_push,
            "push: {expression}"
        );
        assert_eq!(
            static_bool(
                Some(&Value::String(expression.into())),
                &pull_request_inputs
            ),
            expected_on_pull_request,
            "pull_request synchronize: {expression}"
        );
    }
}
