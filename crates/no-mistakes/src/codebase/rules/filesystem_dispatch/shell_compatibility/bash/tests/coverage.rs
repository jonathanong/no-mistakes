use super::super::case_unset::subject_vars_from_header;
use super::super::state::{ArrayState, State, UnanalyzableReason};
use super::super::{analyze, Lexer, Shell};
use crate::fx::FxHashSet;

#[test]
fn global_array_declaration_starts_empty_inside_a_function() {
    let result = analyze(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/shell-compatibility/bash-region-coverage/declare-global-array.sh"
    )));
    assert_eq!(result.findings, vec![("global_items", 6)]);
    assert!(result.unanalyzable.is_empty());
}

#[test]
fn subshell_return_does_not_short_circuit_array_initialization_in_the_function() {
    let result = analyze(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/shell-compatibility/bash-region-coverage/return-in-subshell.sh"
    )));
    assert!(result.findings.is_empty());
    assert!(result.unanalyzable.is_empty());
}

#[test]
fn changed_derivation_weakens_the_state_at_a_join() {
    let mut entry = State::default();
    entry.record_derivation("derived", "original", true);
    let mut incoming = entry.clone();
    assert!(!entry.weakened_by(&incoming));

    incoming.record_derivation("derived", "replacement", true);
    let mut joined = entry.clone();
    joined.merge(&incoming);
    assert!(entry.weakened_by(&joined));
    assert!(joined.derived_from.is_empty());
}

#[test]
fn case_subject_derivation_stops_at_a_repeated_variable() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/shell-compatibility/case-scoped-unset/unresolved.sh"
    ));
    let mut state = State::default();
    state.record_derivation("derived", "original", false);
    state.record_derivation("original", "derived", true);

    let mut matched = false;
    Lexer::new(source, Shell::Bash).commands(|words, _, _| {
        if words.first().is_some_and(|word| word.raw == "case")
            && words.get(1).is_some_and(|word| word.raw == "\"$derived\"")
        {
            let header: Vec<_> = words[1..].iter().collect();
            assert_eq!(
                subject_vars_from_header(&header, &state.derived_from),
                vec![("derived".to_owned(), true), ("original".to_owned(), false)]
            );
            matched = true;
        }
    });
    assert!(matched);
}

#[test]
fn invalidate_drops_derivations_sourced_from_a_touched_name() {
    let mut state = State::default();
    state.record_derivation("kept", "other", true);
    state.record_derivation("dropped", "touched", false);
    let mut touches = FxHashSet::default();
    touches.insert("touched".to_owned());

    state.invalidate(&FxHashSet::default(), &touches);

    assert_eq!(
        state
            .derived_from
            .get("kept")
            .map(|(src, exact)| (src.as_str(), *exact)),
        Some(("other", true))
    );
    assert!(!state.derived_from.contains_key("dropped"));
}

#[test]
fn merge_keeps_an_empty_array_the_other_side_could_not_analyze() {
    let mut left = State::default();
    left.set_known("items", ArrayState::Empty);
    let mut right = State::default();
    right
        .unanalyzable
        .insert("items".to_owned(), UnanalyzableReason::DynamicUnset);

    left.merge(&right);

    assert!(left.arrays.get("items") == Some(&ArrayState::Empty));
    assert!(!left.unanalyzable.contains_key("items"));
}

#[test]
fn unclosed_array_assignment_is_not_a_nounset_expansion() {
    let result = analyze("items=(\n");
    assert!(result.findings.is_empty());
    assert!(result.unanalyzable.is_empty());
}

#[test]
fn review_round_2_branch_coverage() {
    let result = analyze(
        r#"#!/bin/bash
set -u
function fn_if if [[ ${#items[@]} -eq 0 ]]; then exit 1; fi
function fn_arith (( idx++ ))
function fn_extra a b
unset
unset "$dyn"
unset -f -- 'bad-name' "$dyn"
unset foo -- bar
f() { ( unset -f a ); }
cleanup() { unset -f a; unset -f b; }
stop_here() { exit 1; }
drop_stop() { unset -f stop_here; }
if maybe; then drop_stop; fi
items=()
[[ ${#items[@]} -eq 0 ]] && stop_here
printf '%s\n' "${items[@]}"
if [[ ${#items[@]} -eq 0 ]]; then items=(ok); else printf '%s\n' "${items[@]}"; fi
case x in
  a) items=() ;;&
  "${items[@]}") : ;;
esac
printf '%s\n' '$((unclosed' 'unclosed
printf '%s\n' $((1+2
"#,
    );
    assert!(result.findings.iter().any(|(name, _)| *name == "items"));
    assert!(!result.unanalyzable.is_empty());
}

#[test]
fn conditional_and_dynamic_fn_unset_and_substring_condition_deps() {
    let skipped = analyze(
        r#"#!/bin/bash
set +u
enable() { set -u; }
false && unset -f enable
enable
items=()
printf '%s\n' "${items[@]}"
"#,
    );
    assert_eq!(skipped.findings, vec![("items", 7)]);

    let cond_in_fn = analyze(
        r#"#!/bin/bash
set +u
enable() { set -u; }
cleanup() { false && unset -f enable; }
cleanup
enable
items=()
printf '%s\n' "${items[@]}"
"#,
    );
    assert_eq!(cond_in_fn.findings, vec![("items", 8)]);

    let dyn_unset = analyze(
        r#"#!/bin/bash
set -u
stop_here() { exit 1; }
drop_dyn() { unset -f "$name"; }
name=stop_here
drop_dyn
a=()
[[ ${#a[@]} -eq 0 ]] && stop_here
printf '%s\n' "${a[@]}"
"#,
    );
    assert_eq!(dyn_unset.findings, vec![("a", 9)]);

    let substr = analyze(
        r#"#!/bin/bash
set -u
text=ab; offset=0; items=()
[[ ${text:offset:1} == a ]] && items=(x)
offset=1
[[ ${text:offset:1} == a ]] && printf '%s\n' "${items[@]}"
[[ ${text:${idx}:1} == a ]] && printf '%s\n' "${items[@]}"
[[ ${text:offset == a ]] && printf '%s\n' "${items[@]}"
"#,
    );
    assert!(substr.findings.is_empty());
    assert!(substr.unanalyzable.iter().any(|(line, _)| *line == 6));

    let case_chains = analyze(
        r#"#!/bin/bash
set -u
alpha=(1); beta=(2); gamma=(3); keep=(4)
case "$target" in
  alpha) : ;;&
  beta) : ;;&
  gamma) unset "$target" ;;
  *) : ;&
  delta) unset "$target" ;;
esac
case "$target" in
  alpha) exit 1 ;;
esac
case "$target" in
  *) exit 1 ;;
  beta) unset "$target" ;;
esac
printf '%s\n' "${alpha[@]}" "${keep[@]}"
"#,
    );
    assert!(case_chains.findings.iter().any(|(n, _)| *n == "alpha"));

    let empty_to_maybe = analyze(
        r#"#!/bin/bash
set -u
gate=(); items=()
for x in a b; do
  if [[ ${#gate[@]} -gt 0 ]]; then printf '%s\n' "${items[@]}"; fi
  gate=(ready)
done
"#,
    );
    assert_eq!(empty_to_maybe.findings, vec![("items", 5)]);

    let brace_pattern = analyze(
        r#"#!/bin/bash
set -u
items=()
value=$(case x in {) :;; esac)
{ items=(ok); } \
  2>/dev/null/out
{ :; } 2
unset 1bad -- a
f() { ( return ); }
for x in a; do ( break ); break 0; done
break
printf '%s\n' "${items[@]}"
"#,
    );
    assert!(brace_pattern.findings.iter().any(|(n, _)| *n == "items"));
}
