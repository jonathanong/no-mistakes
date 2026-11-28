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

fn advises_at(source: &str, line: usize) {
    let result = analyze(source);
    assert!(
        result.findings.is_empty(),
        "findings={:?} unanalyzable={:?}",
        result.findings,
        result.unanalyzable
    );
    assert!(
        result.unanalyzable.iter().any(|(found, _)| *found == line),
        "{:?}",
        result.unanalyzable
    );
}

#[test]
fn direct_condition_correlation_still_suppresses_the_later_read() {
    let result = analyze(
        r#"#!/bin/bash
set -u
flag=yes
items=()
if [[ $flag == yes ]]; then items=(ready); fi
if [[ $flag == yes ]]; then printf '%s\n' "${items[@]}"; fi
"#,
    );
    assert!(result.findings.is_empty(), "{:?}", result.findings);
}

#[test]
fn indirect_expansion_does_not_reuse_a_fill_fact() {
    advises_at(
        r#"#!/bin/bash
set -u
flag=no
ref=flag
items=()
if [[ "${!ref}" == yes ]]; then items=(ready); fi
flag=yes
if [[ "${!ref}" == yes ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        8,
    );
}

#[test]
fn command_predicate_does_not_reuse_a_fill_fact() {
    advises_at(
        r#"#!/bin/bash
set -u
probe() { [[ $flag == yes ]]; }
flag=no
items=()
if probe; then items=(ready); fi
flag=yes
if probe; then printf '%s\n' "${items[@]}"; fi
"#,
        8,
    );
}

#[test]
fn nested_arithmetic_parentheses_still_invalidate_the_condition() {
    advises_at(
        r#"#!/bin/bash
set -u
n=0
items=()
if [[ $n == 1 ]]; then items=(ready); fi
: "$(( ((0)) + (n=1) ))"
if [[ $n == 1 ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        7,
    );
}

#[test]
fn case_pattern_arithmetic_reaches_the_following_read() {
    advises_at(
        r#"#!/bin/bash
set -u
n=0
items=()
if [[ $n == 1 ]]; then items=(ready); fi
case x in
"$((n=1))") exit 0 ;;
esac
if [[ $n == 1 ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        9,
    );
}

#[test]
fn unset_function_is_not_an_exit() {
    reports(
        r#"#!/bin/bash
set -u
stop_here() { exit 1; }
unset -f stop_here
items=()
[[ ${#items[@]} -eq 0 ]] && stop_here
printf '%s\n' "${items[@]}"
"#,
        "items",
        7,
    );
}

#[test]
fn oversized_break_still_leaves_the_active_loop() {
    reports(
        r#"#!/bin/bash
set -u
items=()
if [[ ${#items[@]} -eq 0 ]]; then
while true; do break 2; done
fi
printf '%s\n' "${items[@]}"
"#,
        "items",
        7,
    );
}

#[test]
fn quoted_break_count_is_a_real_exit() {
    reports(
        r#"#!/bin/bash
set -u
a=()
stop_loop() { while true; do break '1'; done; }
if [[ ${#a[@]} -eq 0 ]]; then stop_loop; fi
printf '%s\n' "${a[@]}"
"#,
        "a",
        6,
    );
}

#[test]
fn conditional_command_inside_a_case_pattern_does_not_swallow_the_file() {
    reports(
        r#"#!/bin/bash
set -u
items=()
value=$(case x in [[) :;; esac)
printf '%s\n' "${items[@]}"
"#,
        "items",
        5,
    );
}

#[test]
fn redirected_compound_keeps_the_skipped_path() {
    reports(
        r#"#!/bin/bash
set -u
items=()
if true; then items=(ready); fi >/dev/null
printf '%s\n' "${items[@]}"
"#,
        "items",
        5,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
if [[ ${#items[@]} -eq 0 ]]; then exit 1; fi >/dev/null
printf '%s\n' "${items[@]}"
"#,
        "items",
        5,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
while true; do items=(ready); break; done >/dev/null
printf '%s\n' "${items[@]}"
"#,
        "items",
        5,
    );
    reports(
        r#"#!/bin/bash
set -u
fill() { items=(ready); } >/dev/null
items=()
fill
printf '%s\n' "${items[@]}"
"#,
        "items",
        6,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
case x in
x) items=(ready) ;;
esac >/dev/null
printf '%s\n' "${items[@]}"
"#,
        "items",
        7,
    );
}

#[test]
fn read_into_an_array_is_uncertain_inside_an_unmodeled_guard() {
    reports(
        r#"#!/bin/bash
set -u
mapfile -t items < "$f"
printf '%s\n' "${items[@]}"
"#,
        "items",
        4,
    );
    advises_at(
        r#"#!/bin/bash
set -u
mapfile -t items < "$f"
if [[ $x == y ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        4,
    );
    advises_at(
        r#"#!/bin/bash
set -u
read -a items < "$f"
if [[ $x == y ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        4,
    );
}

#[test]
fn dead_case_arm_does_not_report_the_entry_array() {
    quiet(
        r#"#!/bin/bash
set -u
items=()
case "${1:-}" in
*) items=(ready) ;;
debug) printf '%s\n' "${items[@]}" ;;
esac
"#,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
case x in
other) : ;;
*) printf '%s\n' "${items[@]}" ;;
esac
"#,
        "items",
        6,
    );
}

#[test]
fn fallthrough_keeps_the_exit_of_the_next_arm() {
    quiet(
        r#"#!/bin/bash
set -u
stop() { case x in *) : ;& *) exit 1 ;; esac; }
items=()
if [[ ${#items[@]} -eq 0 ]]; then stop; fi
printf '%s\n' "${items[@]}"
"#,
    );
    reports(
        r#"#!/bin/bash
set -u
stop() { case x in *) : ;& *) : ;; esac; }
items=()
if [[ ${#items[@]} -eq 0 ]]; then stop; fi
printf '%s\n' "${items[@]}"
"#,
        "items",
        6,
    );
}

#[test]
fn unary_file_test_does_not_reuse_a_fill_fact() {
    advises_at(
        r#"#!/bin/bash
set -u
items=()
if [ -a "$path" ]; then items=(ready); fi
if [ -a "$path" ]; then printf '%s\n' "${items[@]}"; fi
"#,
        5,
    );
    advises_at(
        r#"#!/bin/bash
set -u
items=()
if test -a "$path"; then items=(ready); fi
if test -a "$path"; then printf '%s\n' "${items[@]}"; fi
"#,
        5,
    );
}

#[test]
fn plain_unset_removes_a_function_exit_without_a_variable() {
    reports(
        r#"#!/bin/bash
set -u
stop_here() { exit 1; }
unset stop_here
items=()
[[ ${#items[@]} -eq 0 ]] && stop_here
printf '%s\n' "${items[@]}"
"#,
        "items",
        7,
    );
    quiet(
        r#"#!/bin/bash
set -u
stop_here() { exit 1; }
stop_here=()
unset stop_here
items=()
[[ ${#items[@]} -eq 0 ]] && stop_here
printf '%s\n' "${items[@]}"
"#,
    );
    quiet(
        r#"#!/bin/bash
set -u
stop_here() { exit 1; }
unset -v stop_here
items=()
[[ ${#items[@]} -eq 0 ]] && stop_here
printf '%s\n' "${items[@]}"
"#,
    );
}

#[test]
fn user_function_hides_a_builtin_exit() {
    reports(
        r#"#!/bin/bash
exit() { :; }
set -u
items=()
[[ ${#items[@]} -eq 0 ]] && exit
printf '%s\n' "${items[@]}"
"#,
        "items",
        6,
    );
    quiet(
        r#"#!/bin/bash
set -u
items=()
[[ ${#items[@]} -eq 0 ]] && exit 1
printf '%s\n' "${items[@]}"
"#,
    );
}
