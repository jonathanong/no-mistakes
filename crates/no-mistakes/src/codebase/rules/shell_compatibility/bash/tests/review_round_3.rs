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

#[test]
fn mapfile_callback_effects_and_zero_input_safety() {
    reports(
        "set -u\nitems=(x)\nwipe() { items=(); }\nmapfile -c1 -C wipe lines <<< x\n: \"${items[@]}\"\n",
        "items",
        5,
    );
    reports(
        "set -u\nitems=()\nfill() { items=(x); }\nmapfile -c1 -C fill lines < /dev/null\n: \"${items[@]}\"\n",
        "items",
        5,
    );
    let res =
        analyze("set -u\nitems=(x)\nmapfile -c1 -C 'items=()' lines <<< x\n: \"${items[@]}\"\n");
    assert!(res.findings.is_empty());
    assert_eq!(res.unanalyzable.len(), 1);
    assert_eq!(res.unanalyzable[0].0, 4);
}

#[test]
fn repeated_local_declaration_preserves_existing_local_but_shadows_outer() {
    quiet("set -u\nf() {\n  local -a items=(x)\n  local -a items\n  : \"${items[@]}\"\n}\nf\n");
    reports(
        "set -u\nitems=(x)\nf() {\n  local -a items\n  : \"${items[@]}\"\n}\nf\n",
        "items",
        5,
    );
    reports(
        "set -u\nitems=(x)\nbad=()\nf() {\n  if cond; then\n    local -a items=(y)\n  fi\n  local -a items\n  if [[ ${#items[@]} -gt 0 ]]; then\n    : \"${bad[@]}\"\n  fi\n}\nf\n",
        "bad",
        10,
    );
}

#[test]
fn redirected_set_preserves_active_nounset() {
    reports(
        "set -u\nitems=()\nset +u > /nonexistent/path\n: \"${items[@]}\"\n",
        "items",
        4,
    );
}

#[test]
fn typeset_array_assignment_updates_state() {
    reports(
        "set -u\ngate=()\nbad=()\ntypeset -a gate=(x)\nif [[ ${#gate[@]} -gt 0 ]]; then\n  : \"${bad[@]}\"\nfi\n",
        "bad",
        6,
    );
    reports(
        "set -u\nitems=(x)\ntypeset -a items=()\n: \"${items[@]}\"\n",
        "items",
        4,
    );
}

#[test]
fn indexed_case_pattern_unset_widens_instead_of_emptying() {
    let res = analyze(
        "set -u\ngate=(x y)\nbad=()\ncase $target in\n  'gate[0]')\n    unset \"$target\"\n    if [[ ${#gate[@]} -gt 0 ]]; then\n      : \"${bad[@]}\"\n    fi\n    ;;\nesac\n",
    );
    assert!(res.findings.iter().any(|&(n, l)| n == "bad" && l == 8));
}

#[test]
fn indexed_assignment_subscript_arithmetic_invalidates_condition() {
    let res = analyze(
        "set -u\nn=0\nitems=()\nif [[ $n -eq 0 ]]; then\n  items=(x)\nfi\narr[n++]=1\nif [[ $n -eq 0 ]]; then\n  : \"${items[@]}\"\nfi\n",
    );
    assert!(res.findings.is_empty());
    assert_eq!(res.unanalyzable.len(), 1);
    assert_eq!(res.unanalyzable[0].0, 9);
}

#[test]
fn for_loop_with_literal_item_populates_loop_var_in_body() {
    reports(
        "set -u\ngate=()\nbad=()\nfor gate in a b; do\n  if [[ ${#gate[@]} -gt 0 ]]; then\n    : \"${bad[@]}\"\n  fi\ndone\n",
        "bad",
        6,
    );
    reports(
        "set -u\nitems=()\nfor items in a \"${items[@]}\"; do\n  : \"${items[@]}\"\ndone\n",
        "items",
        3,
    );
}

#[test]
fn read_array_mode_ignores_positional_scalars_and_subscripts() {
    quiet(
        "set -u\ngate=()\nbad=()\nread -a arr gate < input\nif [[ ${#gate[@]} -gt 0 ]]; then\n  : \"${bad[@]}\"\nfi\n",
    );
    quiet(
        "set -u\ngate=()\nbad=()\nread -a 'gate[0]' < input\nif [[ ${#gate[@]} -gt 0 ]]; then\n  : \"${bad[@]}\"\nfi\n",
    );
    reports(
        "set -u\ngate=()\nbad=()\nread 'gate[0]' < input\nif [[ ${#gate[@]} -gt 0 ]]; then\n  : \"${bad[@]}\"\nfi\n",
        "bad",
        6,
    );
}

#[test]
fn nested_multiline_double_parens_stay_linear() {
    let mut script = String::from("#!/bin/bash\nset -u\n");
    for _ in 0..400 {
        script.push_str("((\n");
    }
    script.push_str("1\n");
    for _ in 0..400 {
        script.push_str("))\n");
    }
    let _ = analyze(&script);
}

#[test]
fn untracked_copy_source_stays_uncertain_not_empty() {
    reports(
        "set -u\nsource[0]=x\ngate=(\"${source[@]}\")\nbad=()\nif [[ ${#gate[@]} -gt 0 ]]; then : \"${bad[@]}\"; fi\n",
        "bad",
        5,
    );
}

#[test]
fn forward_referenced_function_effects_resolve_at_call_site() {
    reports(
        "outer() { turn_on_nounset; }\nturn_on_nounset() { set -u; }\na=()\nouter\nprintf '%s\\n' \"${a[@]}\"\n",
        "a",
        5,
    );
    quiet(
        "outer() { turn_on_nounset; }\na=()\nouter\nturn_on_nounset() { set -u; }\nprintf '%s\\n' \"${a[@]}\"\n",
    );
    reports(
        "set -u\nitems=(x)\nouter() { wipe; }\nwipe() { items=(); }\nouter\n: \"${items[@]}\"\n",
        "items",
        6,
    );
    reports(
        "turn_on_nounset() { deep; }\nouter() { turn_on_nounset; }\ndeep() { set -u; }\na=()\nouter\n: \"${a[@]}\"\n",
        "a",
        6,
    );
}

#[test]
fn builtin_unset_removes_function_exit_summary() {
    reports(
        "set -u\nstop_here() { exit 1; }\na=()\nbuiltin unset -f stop_here\n[[ ${#a[@]} -eq 0 ]] && stop_here\n: \"${a[@]}\"\n",
        "a",
        6,
    );
}

#[test]
fn brace_group_on_left_of_and_or_keeps_writes() {
    quiet("set -u\na=()\n{ a=(ready); } && :\nprintf '%s\\n' \"${a[@]}\"\n");
    quiet("set -u\na=()\n{ a=(ready); } || :\nprintf '%s\\n' \"${a[@]}\"\n");
    reports(
        "set -u\na=()\n{ a=(ready); } | cat\nprintf '%s\\n' \"${a[@]}\"\n",
        "a",
        4,
    );
}

#[test]
fn quoted_zero_in_numeric_length_guard_protects_expansion() {
    quiet("set -u\na=()\n[ \"${#a[@]}\" -eq '0' ] && exit\nprintf '%s\\n' \"${a[@]}\"\n");
    quiet("set -u\na=()\n[[ \"${#a[@]}\" -gt \"0\" ]] || exit\nprintf '%s\\n' \"${a[@]}\"\n");
    reports(
        "set -u\na=()\n[ \"${#a[@]}\" -eq \"$z\" ] && exit\nprintf '%s\\n' \"${a[@]}\"\n",
        "a",
        4,
    );
}

#[test]
fn case_insensitive_unset_keeps_other_case_live() {
    reports(
        "set -u\nITEMS=(x)\nbad=()\nnormalized=\"${target,,}\"\ncase \"$normalized\" in\n  items) unset \"$target\" ;;\nesac\nif [[ ${#ITEMS[@]} -gt 0 ]]; then\n  : \"${bad[@]}\"\nfi\n",
        "bad",
        9,
    );
}

#[test]
fn coproc_bodies_do_not_promote_parent_facts() {
    reports(
        "set -u\nitems=()\ncoproc {\n  [[ ${#items[@]} -gt 0 ]] || exit 1\n}\nprintf '%s\\n' \"${items[@]}\"\n",
        "items",
        6,
    );
    reports(
        "set -u\nitems=()\ncoproc MYPROC {\n  [[ ${#items[@]} -gt 0 ]] || exit 1\n}\nprintf '%s\\n' \"${items[@]}\"\n",
        "items",
        6,
    );
    reports(
        "set -u\nitems=()\ncoproc if true; then\n  [[ ${#items[@]} -gt 0 ]] || exit 1\nfi\nprintf '%s\\n' \"${items[@]}\"\n",
        "items",
        6,
    );
}
