use super::{static_bool, InputState, StaticBool, StaticValue};
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
fn bracketed_github_event_name_access_matches_dot_access() {
    let inputs = InputState::from([(
        "\0github.event_name".into(),
        StaticValue::String("push".into()),
    )]);
    for (expression, expected) in [
        ("github.event_name == 'push'", StaticBool::True),
        ("github['event_name'] == 'push'", StaticBool::True),
        (
            "GITHUB [ 'EVENT_NAME' ] == 'pull_request'",
            StaticBool::False,
        ),
        ("github[\"event_name\"] == 'push'", StaticBool::Unknown),
        ("github['event_name'].nested == 'push'", StaticBool::Unknown),
    ] {
        assert_eq!(
            static_bool(Some(&Value::String(expression.into())), &inputs),
            expected,
            "{expression}"
        );
    }
}

#[test]
fn bracketed_github_event_action_access_matches_dot_access() {
    let inputs = InputState::from([(
        "\0github.event.action".into(),
        StaticValue::String("opened".into()),
    )]);
    for (expression, expected) in [
        ("github.event.action == 'opened'", StaticBool::True),
        ("github.event['action'] == 'CLOSED'", StaticBool::False),
        ("GITHUB.EVENT [ 'ACTION' ] != 'closed'", StaticBool::True),
        ("github.event[\"action\"] == 'opened'", StaticBool::Unknown),
        (
            "github.event['action'].nested == 'opened'",
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
