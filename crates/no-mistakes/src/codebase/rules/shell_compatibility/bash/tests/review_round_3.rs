use super::analyze;

fn reports(source: &str, name: &str, line: usize) {
    let result = analyze(source);
    assert!(
        result
            .findings
            .iter()
            .any(|(found, found_line)| *found == name && *found_line == line),
        "findings={:?} unanalyzable={:?}",
        result.findings,
        result.unanalyzable
    );
}

fn quiet(source: &str) {
    let result = analyze(source);
    assert!(
        result.findings.is_empty() && result.unanalyzable.is_empty(),
        "findings={:?} unanalyzable={:?}",
        result.findings,
        result.unanalyzable
    );
}

#[test]
fn unsetting_one_element_keeps_the_length_guard_body_live() {
    reports(
        "set -u\ngate=(x y)\nbad=()\nunset 'gate[0]'\nif [[ ${#gate[@]} -gt 0 ]]; then\n  printf '%s\\n' \"${bad[@]}\"\nfi\n",
        "bad",
        6,
    );
}

#[test]
fn unsetting_the_whole_array_still_kills_the_length_guard_body() {
    quiet(
        "set -u\ngate=(x y)\nbad=()\nunset gate\nif [[ ${#gate[@]} -gt 0 ]]; then\n  printf '%s\\n' \"${bad[@]}\"\nfi\n",
    );
}

#[test]
fn case_arm_with_two_unset_targets_does_not_empty_either() {
    reports(
        "set -u\nleft=(x)\nright=(y)\nbad=()\ncase $1 in\n  left|right) unset \"$1\" ;;\nesac\nif [[ ${#left[@]} -gt 0 ]]; then\n  printf '%s\\n' \"${bad[@]}\"\nfi\n",
        "bad",
        9,
    );
}

#[test]
fn case_arm_with_a_glob_pattern_is_not_a_literal_unset_target() {
    reports(
        "set -u\nleft=(x)\nbad=()\ncase $1 in\n  left|l*) unset \"$1\" ;;\nesac\nif [[ ${#left[@]} -gt 0 ]]; then\n  printf '%s\\n' \"${bad[@]}\"\nfi\n",
        "bad",
        8,
    );
}

#[test]
fn scalar_read_into_an_empty_array_name_leaves_it_uncertain() {
    reports(
        "set -u\ngate=()\nbad=()\nread gate < input\nif [[ ${#gate[@]} -gt 0 ]]; then\n  printf '%s\\n' \"${bad[@]}\"\nfi\n",
        "bad",
        6,
    );
}

#[test]
fn declare_print_does_not_declare_a_local() {
    reports(
        "set -u\nitems=()\nf() {\n  declare -p items\n  printf '%s\\n' \"${items[@]}\"\n}\nf\n",
        "items",
        5,
    );
}
