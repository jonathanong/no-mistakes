use super::*;
use std::path::PathBuf;
#[path = "tests/append_aliases.rs"]
mod append_aliases;
#[path = "tests/append_sites.rs"]
mod append_sites;
#[path = "tests/branch_correlation.rs"]
mod branch_correlation;
#[path = "tests/defensive.rs"]
mod defensive;
#[path = "tests/equivalent.rs"]
mod equivalent;
#[path = "tests/helper_effects.rs"]
mod helper_effects;
#[path = "tests/helpers.rs"]
mod helpers;
#[path = "tests/metadata.rs"]
mod metadata;
#[path = "tests/positions.rs"]
mod positions;

fn file_facts(name: &str) -> EmbeddedSqlFileFacts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    extract_embedded_sql_from_source(&path, &source, &options)
}
fn calls(name: &str) -> Vec<EmbeddedSqlCall> {
    file_facts(name).calls
}
fn texts(call: &EmbeddedSqlCall) -> Vec<&str> {
    call.variants
        .iter()
        .map(|version| version.sql_text.as_str())
        .collect()
}

#[test]
fn executor_conditionals_keep_one_dynamic_call_with_complete_alternatives() {
    let calls = calls("variants-conditional.ts");
    assert_eq!(calls.len(), 2);
    assert_eq!(texts(&calls[0]), ["SELECT 1", "SELECT 2"]);
    assert_eq!(texts(&calls[1]), ["SELECT sql_placeholder_1", "SELECT 3"]);
    assert!(calls
        .iter()
        .all(|call| call.kind == EmbeddedSqlKind::Dynamic));
}
#[test]
fn logical_operators_respect_string_truthiness_nullishness_and_absent_fragments() {
    let calls = calls("variants-logical.ts");
    assert_eq!(texts(&calls[0]), ["SELECT 1", "SELECT 2"]);
    assert_eq!(texts(&calls[1]), ["SELECT 3", "SELECT 4"]);
    assert_eq!(texts(&calls[2]), ["SELECT 6"]);
    assert_eq!(texts(&calls[3]), ["SELECT 1 ", "SELECT 1 LIMIT 1"]);
}
#[test]
fn mutations_keep_taken_and_not_taken_paths() {
    let calls = calls("variants-if.ts");
    assert_eq!(
        texts(&calls[0]),
        ["SELECT id FROM users LIMIT 1", "SELECT id FROM users"]
    );
    assert_eq!(
        texts(&calls[1]),
        [
            "SELECT id FROM users LIMIT 2",
            "SELECT id FROM users LIMIT 3"
        ]
    );
    assert_eq!(
        texts(&calls[2]),
        ["SELECT id FROM users", "SELECT id FROM users LIMIT 4"]
    );
}
#[test]
fn switch_keeps_default_no_match_and_fallthrough_paths() {
    let calls = calls("variants-switch.ts");
    assert_eq!(
        texts(&calls[0]),
        ["SELECT 1 /* a */", "SELECT 1 /* b */", "SELECT 1 /* c */"]
    );
    let mut fall = texts(&calls[1]);
    fall.sort();
    assert_eq!(
        fall,
        ["SELECT 2", "SELECT 2 /* a */ /* b */", "SELECT 2 /* b */"]
    );
}
#[test]
fn branch_assignments_do_not_invent_cross_path_combinations() {
    let calls = calls("variants-bindings.ts");
    assert_eq!(texts(&calls[0]), ["SELECT 1", "SELECT 2"]);
    assert_eq!(texts(&calls[1]), ["SELECT 1", "SELECT 20"]);
}
#[test]
fn nested_fragments_raw_literals_and_literal_joins_are_spliced() {
    let calls = calls("variants-nested.ts");
    assert_eq!(
        texts(&calls[0]),
        [
            "SELECT id FROM users WHERE active LIMIT 1",
            "SELECT id FROM users WHERE deleted LIMIT 1"
        ]
    );
    assert_eq!(
        texts(&calls[1]),
        ["SELECT id FROM users WHERE active LIMIT 1"]
    );
    assert_eq!(
        texts(&calls[2]),
        ["SELECT id FROM users WHERE active AND enabled LIMIT 1"]
    );
    assert_eq!(
        texts(&calls[3]),
        [
            "SELECT id FROM users WHERE active LIMIT 1",
            "SELECT id FROM users sql_placeholder_1 LIMIT 1"
        ]
    );
}
#[test]
fn opaque_alternatives_loops_cross_file_helpers_depth_and_cap_fail_closed() {
    for name in ["variants-opaque.ts", "variants-cap.ts"] {
        let calls = calls(name);
        assert!(!calls.is_empty());
        assert!(
            calls
                .iter()
                .all(|call| call.kind == EmbeddedSqlKind::Dynamic && call.variants.is_empty()),
            "{name}: {calls:#?}"
        );
    }
    assert_eq!(MAX_EMBEDDED_SQL_VARIANTS, 16);
}
#[test]
fn nonbranching_calls_retain_their_legacy_fields_and_statement_projection_bytes() {
    let calls = calls("variants-single.ts");
    assert!(calls.iter().all(|call| call.variants.is_empty()));
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert_eq!(calls[1].kind, EmbeddedSqlKind::ImmutableLocal);
    assert_eq!(calls[2].kind, EmbeddedSqlKind::Dynamic);
    let mut snapshot = String::new();
    for call in &calls {
        let original =
            crate::codebase::postgres::extract_sql_statement_facts_for_embedded_call(call);
        let projected = crate::codebase::postgres::extract_sql_statement_facts_for_embedded_call(
            &call.statement_calls().next().unwrap(),
        );
        assert_eq!(
            format!("{original:?}").into_bytes(),
            format!("{projected:?}").into_bytes()
        );
        // The baseline was generated with the pre-variant public extractor.
        snapshot.push_str(&format!(
            "{:?}\n{original:?}\n",
            (
                call.line,
                &call.callee,
                &call.sql_text,
                call.kind,
                call.declaration_line,
                &call.sql_source_positions,
                &call.recovered_placeholder_positions
            )
        ));
    }
    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/variants-single-legacy.txt");
    assert_eq!(snapshot.as_bytes(), std::fs::read(baseline).unwrap());
}

#[test]
fn statically_skipped_arms_do_not_poison_recovery_or_invent_combinations() {
    let calls = calls("variants-truthiness.ts");
    for (call, expected) in calls.iter().take(8).zip([
        "SELECT 1",
        "SELECT 2",
        "SELECT 3",
        "SELECT 4",
        "SELECT 5 LIMIT 1",
        "SELECT 6 ",
        "SELECT 7",
        "SELECT 8",
    ]) {
        assert_eq!(texts(call), [expected]);
    }
    assert_eq!(texts(&calls[8]), ["SELECT 10;", "SELECT 9"]);
    assert_eq!(texts(&calls[9]), ["SELECT 11"]);
}

#[test]
fn exactly_sixteen_versions_are_retained_in_deterministic_order() {
    let calls = calls("variants-cap-boundary.ts");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].variants.len(), MAX_EMBEDDED_SQL_VARIANTS);
    assert_eq!(
        texts(&calls[0]).first().copied(),
        Some("SELECT 1 /* a1 */ /* b1 */ /* c1 */ /* d1 */")
    );
    assert_eq!(
        texts(&calls[0]).last().copied(),
        Some("SELECT 1 /* a0 */ /* b0 */ /* c0 */ /* d0 */")
    );
    assert_eq!(calls, self::calls("variants-cap-boundary.ts"));
}

#[test]
fn switch_fallthrough_retains_correlation_between_bindings() {
    let calls = calls("variants-switch-correlated.ts");
    let mut actual = texts(&calls[0]);
    actual.sort();
    assert_eq!(actual, ["SELECT 2", "SELECT a2", "SELECT z9"]);
}

#[test]
fn aliases_use_the_binding_and_tag_visible_in_their_lexical_scope() {
    let calls = calls("variants-shadowing.ts");
    assert_eq!(calls.len(), 4);
    assert_eq!(
        texts(&calls[0]),
        ["SELECT id FROM users WHERE id = sql_placeholder_1"]
    );
    // File-wide fragment detection is conservative; the scoped scalar remains a bind.
    assert_eq!(
        texts(&calls[1]),
        ["SELECT id FROM users WHERE name = sql_placeholder_1"]
    );
    assert!(calls[2].is_unanalyzable());
    assert_eq!(texts(&calls[3]), ["SELECT id FROM users WHERE active"]);
}

#[test]
fn unknown_paths_and_cross_function_or_repeated_mutations_remain_opaque() {
    let calls = calls("variants-opaque-scopes.ts");
    assert_eq!(calls.len(), 5);
    assert!(
        calls.iter().all(EmbeddedSqlCall::is_unanalyzable),
        "{calls:#?}"
    );
}

#[test]
fn assignment_rhs_call_observes_the_previous_binding_before_it_is_invalidated() {
    let calls = calls("variants-assignment-order.ts");
    assert_eq!(calls.len(), 2);
    assert_eq!(texts(&calls[0]), ["SELECT 1", "SELECT 2"]);
    assert!(calls[1].is_unanalyzable());
}

#[test]
fn literal_switches_select_only_the_reachable_case_and_its_fallthrough() {
    let calls = calls("variants-literal-switch.ts");
    assert_eq!(calls.len(), 5);
    for (call, expected) in calls.iter().zip([
        "SELECT 1",
        "SELECT defaultb",
        "SELECT number",
        "SELECT true",
        "SELECT 4",
    ]) {
        assert_eq!(texts(call), [expected]);
    }
}

#[test]
fn unsupported_abrupt_control_does_not_publish_unreachable_statement_versions() {
    let calls = calls("variants-abrupt.ts");
    assert_eq!(calls.len(), 3);
    assert!(
        calls.iter().all(EmbeddedSqlCall::is_unanalyzable),
        "{calls:#?}"
    );
}

#[test]
fn calls_inside_branches_see_only_the_paths_that_reach_their_physical_call_site() {
    let calls = calls("variants-branch-calls.ts");
    assert_eq!(calls.len(), 5);
    let mut before_append = texts(&calls[0]);
    before_append.sort();
    assert_eq!(before_append, ["SELECT 1", "SELECT 1 /* a */"]);
    let mut after_append = texts(&calls[1]);
    after_append.sort();
    assert_eq!(
        after_append,
        ["SELECT 1 /* a */ /* b */", "SELECT 1 /* b */"]
    );
    assert_eq!(texts(&calls[2]), ["SELECT 1"]);
    assert_eq!(texts(&calls[3]), ["SELECT 2"]);
    assert_eq!(texts(&calls[4]), ["SELECT 3"]);
}

#[test]
fn static_builder_calls_joins_and_fluent_branch_fragments_are_complete_versions() {
    let calls = calls("variants-builders.ts");
    assert_eq!(calls.len(), 9);
    assert_eq!(texts(&calls[0]), ["SELECT id FROM users "]);
    assert_eq!(
        texts(&calls[1]),
        ["SELECT id FROM users WHERE active AND enabled"]
    );
    assert_eq!(texts(&calls[2]), ["SELECT id FROM users WHERE active"]);
    assert_eq!(
        texts(&calls[3]),
        [
            "SELECT id FROM users WHERE active AND enabled",
            "SELECT id FROM users WHERE active AND verified"
        ]
    );
    assert_eq!(
        texts(&calls[4]),
        [
            "SELECT id FROM users WHERE active AND enabled",
            "SELECT id FROM users WHERE active OR enabled"
        ]
    );
    assert_eq!(texts(&calls[5]), ["SELECT id FROM users WHERE active"]);
    assert_eq!(texts(&calls[6]), ["SELECT id FROM users WHERE active"]);
    assert_eq!(
        texts(&calls[7]),
        ["SELECT id FROM users WHERE active AND enabled"]
    );
    assert_eq!(
        texts(&calls[8]),
        ["SELECT id FROM users WHERE active AND enabled"]
    );
}

#[test]
fn nullable_alternatives_use_the_fallback_but_null_executor_arguments_stay_opaque() {
    let calls = calls("variants-nullish.ts");
    assert_eq!(calls.len(), 6);
    for (call, mut expected) in calls.iter().take(3).zip([
        vec!["SELECT 1", "SELECT 2"],
        vec!["SELECT sql_placeholder_1", "SELECT 3"],
        vec![
            "SELECT id FROM users WHERE active",
            "SELECT id FROM users WHERE deleted",
        ],
    ]) {
        let mut actual = texts(call);
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
    }
    assert!(calls[3..].iter().all(EmbeddedSqlCall::is_unanalyzable));
}

#[test]
fn trusted_builder_conditions_are_truthy_even_when_their_sql_is_empty() {
    let calls = calls("variants-builder-truthiness.ts");
    assert_eq!(calls.len(), 4);
    for (call, expected) in calls.iter().zip([
        "SELECT 1 LIMIT 1",
        "SELECT 2 LIMIT 2",
        "SELECT 3 LIMIT 3",
        "SELECT 4 LIMIT 4",
    ]) {
        assert_eq!(texts(call), [expected]);
    }
}

#[test]
fn repeated_immutable_conditions_keep_expression_and_mutation_choices_correlated() {
    let calls = calls("variants-correlated-conditions.ts");
    assert_eq!(calls.len(), 5);
    for call in &calls[..2] {
        let mut actual = texts(call);
        actual.sort();
        assert_eq!(actual, ["SELECT 10", "SELECT 29"]);
    }
    let mut actual = texts(&calls[2]);
    actual.sort();
    assert_eq!(actual, ["SELECT 10", "SELECT 2"]);
    for call in &calls[3..] {
        let mut actual = texts(call);
        actual.sort();
        assert_eq!(actual, ["SELECT 19", "SELECT 20"]);
    }
}

#[test]
fn scalar_falsy_arms_are_binds_while_untaken_fragment_guards_are_absent() {
    let calls = calls("variants-falsy-bindings.ts");
    assert_eq!(calls.len(), 11);
    for (call, expected) in calls
        .iter()
        .take(3)
        .zip(["SELECT 1", "SELECT 2", "SELECT 3"])
    {
        assert_eq!(texts(call), ["SELECT sql_placeholder_1", expected]);
        assert_eq!(call.variants[0].recovered_placeholder_positions.len(), 1);
        assert!(call.variants[1].recovered_placeholder_positions.is_empty());
    }
    for (call, expected) in calls[3..6]
        .iter()
        .zip(["SELECT 4 ", "SELECT 5 ", "SELECT 6 "])
    {
        assert_eq!(texts(call), [expected]);
        assert!(call.variants[0].recovered_placeholder_positions.is_empty());
    }
    let mut optional = texts(&calls[6]);
    optional.sort();
    assert_eq!(optional, ["SELECT 7 ", "SELECT 7 LIMIT 1"]);
    assert!(calls[7..].iter().all(EmbeddedSqlCall::is_unanalyzable));
}

#[path = "tests/logical.rs"]
mod logical;
