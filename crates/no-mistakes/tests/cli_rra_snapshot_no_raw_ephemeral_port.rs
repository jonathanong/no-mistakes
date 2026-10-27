use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "no-raw-ephemeral-port";

#[test]
fn no_raw_ephemeral_port_json_matches_snapshot() {
    let empty = rules_path(RULE, "configs", "empty.yml");
    let include_py = rules_path(RULE, "configs", "include-py.yml");
    assert_saved_fixture(
        "no_raw_ephemeral_port",
        &[
            (
                "python_shell_yaml",
                rules_path(RULE, "fixture", "fail-python"),
                empty.clone(),
            ),
            (
                "listen_zero",
                rules_path(RULE, "fixture", "fail-listen"),
                empty.clone(),
            ),
            ("pass", rules_path(RULE, "fixture", "pass"), empty.clone()),
            (
                "allow_binder",
                rules_path(RULE, "fixture", "allow"),
                rules_path(RULE, "configs", "allow-binder.yml"),
            ),
            (
                "custom_message",
                rules_path(RULE, "fixture", "fail-listen"),
                rules_path(RULE, "configs", "custom-message.yml"),
            ),
            (
                "include_py_on_listen",
                rules_path(RULE, "fixture", "fail-listen"),
                include_py.clone(),
            ),
            (
                "include_miss",
                rules_path(RULE, "fixture", "fail-python"),
                rules_path(RULE, "configs", "include-nope.yml"),
            ),
            (
                "disable_file",
                rules_path(RULE, "fixture", "disable"),
                empty.clone(),
            ),
            (
                "next_line_disable",
                rules_path(RULE, "fixture", "next-line-disable"),
                empty.clone(),
            ),
            (
                "same_line_bind",
                rules_path(RULE, "fixture", "same-line-bind"),
                empty.clone(),
            ),
            (
                "js_function_bind",
                rules_path(RULE, "fixture", "js-function-bind"),
                empty.clone(),
            ),
            (
                "js_bind_listen",
                rules_path(RULE, "fixture", "js-bind-listen"),
                empty.clone(),
            ),
            (
                "literal_recall",
                rules_path(RULE, "fixture", "literal-recall"),
                empty,
            ),
            (
                "literal_recall_py",
                rules_path(RULE, "fixture", "literal-recall"),
                include_py,
            ),
            (
                "literal_recall_allow_ts",
                rules_path(RULE, "fixture", "literal-recall"),
                rules_path(RULE, "configs", "allow-ts.yml"),
            ),
        ],
    );
}
