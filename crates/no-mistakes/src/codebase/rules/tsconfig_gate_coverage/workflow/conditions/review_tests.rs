use super::{evaluation::static_bool, InputState, StaticBool, StaticValue};
use serde_yaml::Value;

#[test]
fn unsupported_case_functions_remain_unknown() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("case(true, false, true)", StaticBool::Unknown),
        ("case(false, true, true, false, true)", StaticBool::Unknown),
        (
            "case(false, true, true, 'release', false)",
            StaticBool::Unknown,
        ),
        ("case(true, false, github.ref)", StaticBool::Unknown),
        (
            "case(false, 'ignored', 'release') == 'release'",
            StaticBool::Unknown,
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
fn join_functions_resolve_static_collections_before_comparisons() {
    let inputs = InputState::new();
    for (expression, expected) in [
        ("join(fromJSON('[\"a\"]'), ',') == 'b'", StaticBool::False),
        (
            "join(fromJSON('[\"a\", \"b\"]'), '-') == 'a-b'",
            StaticBool::True,
        ),
        (
            "join(fromJSON('[\"a\", {}]'), ',') == 'a,'",
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
fn exact_ref_comparisons_respect_known_exclusion_only_constraints() {
    let inputs = InputState::from([(
        "\0github.ref.exclusions".into(),
        StaticValue::Sequence(vec![StaticValue::String("refs/heads/main".into())]),
    )]);

    assert_eq!(
        static_bool(
            Some(&Value::String("github.ref == 'refs/heads/main'".into())),
            &inputs
        ),
        StaticBool::False
    );
    assert_eq!(
        static_bool(
            Some(&Value::String("github.ref == 'refs/heads/dev'".into())),
            &inputs
        ),
        StaticBool::Unknown
    );
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
