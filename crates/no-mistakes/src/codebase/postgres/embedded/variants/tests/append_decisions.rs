use super::*;

#[test]
fn append_budget_retains_singleton_decisions_with_multiple_calls_and_reversed_guards() {
    let calls = calls("variants-append-decisions.ts");
    let grouped = &calls[0];
    let base = "SELECT 1 /* common-a */ /* common-b */";
    assert_eq!(grouped.variants.len(), 21);
    assert_eq!(grouped.variants[0].sql_text, base);
    for index in 0..19 {
        assert_eq!(
            grouped.variants[index + 1].sql_text,
            format!("{base} /* g{index} */")
        );
    }
    assert_eq!(
        grouped.variants[20].sql_text,
        format!("{base} /* last-a */ /* last-b */")
    );
    assert!(grouped.variants_truncated());
}

#[test]
fn nested_noop_paths_do_not_displace_distinct_singleton_sql() {
    let calls = calls("variants-append-decisions.ts");
    let nested = &calls[1];
    let texts = texts(nested);
    assert_eq!(texts[0], "SELECT 2");
    for index in 0..10 {
        assert!(
            texts.contains(&format!("SELECT 2 /* outer{index} */").as_str()),
            "{texts:#?}"
        );
        assert!(
            texts.contains(&format!("SELECT 2 /* outer{index} */ /* inner{index} */").as_str()),
            "{texts:#?}"
        );
    }
    let final_singleton = texts
        .iter()
        .position(|sql| *sql == "SELECT 2 /* final-a */ /* final-b */")
        .unwrap();
    assert!(texts[..final_singleton]
        .iter()
        .all(|sql| !sql.contains("inner")));
    assert!(nested.variants_truncated());
    assert!(nested.variants.len() <= 22);
}

#[test]
fn one_loop_iteration_with_multiple_appends_is_one_breadth_decision() {
    let calls = calls("variants-append-decisions.ts");
    let repeated = &calls[2];
    assert_eq!(repeated.variants.len(), 21);
    assert_eq!(repeated.variants[0].sql_text, "SELECT 3");
    for index in 0..19 {
        assert_eq!(
            repeated.variants[index + 1].sql_text,
            format!("SELECT 3 /* r{index} */")
        );
    }
    assert_eq!(
        repeated.variants[20].sql_text,
        "SELECT 3 /* loop-a */ /* loop-b */"
    );
    assert!(repeated.variants_truncated());
}

#[test]
fn nested_decision_keeps_its_minimal_feasible_parent_witness_before_unrelated_pairs() {
    let calls = calls("variants-append-decisions.ts");
    let nested = &calls[3];
    assert_eq!(nested.variants.len(), 22);
    assert_eq!(nested.variants[0].sql_text, "SELECT 4");
    for index in 0..19 {
        assert_eq!(
            nested.variants[index + 1].sql_text,
            format!("SELECT 4 /* p{index} */")
        );
    }
    assert_eq!(nested.variants[20].sql_text, "SELECT 4 /* parent */");
    assert_eq!(
        nested.variants[21].sql_text,
        "SELECT 4 /* parent */ /* child */"
    );
    assert!(nested.variants_truncated());
}

#[test]
fn conditional_suffixes_keep_off_and_every_singleton_in_source_order() {
    let calls = calls("variants-append-conditional-suffix.ts");
    let call = &calls[0];
    assert_eq!(call.variants.len(), 21);
    assert_eq!(call.variants[0].sql_text, "SELECT 10");
    for index in 0..19 {
        assert_eq!(
            call.variants[index + 1].sql_text,
            format!("SELECT 10 /* s{index} */")
        );
    }
    assert_eq!(call.variants[20].sql_text, "SELECT 10 /* last */");
    assert!(call.variants_truncated());
}

#[test]
fn nested_conditional_suffix_keeps_parent_and_inner_witnesses_over_earlier_pairs() {
    let calls = calls("variants-append-conditional-suffix.ts");
    let call = &calls[1];
    let texts = texts(call);
    assert_eq!(texts.len(), 22);
    assert!(texts.contains(&"SELECT 11 /* outer */"));
    assert!(texts.contains(&"SELECT 11 /* outer */ /* inner */"));
    assert!(!texts.contains(&"SELECT 11 /* inner */"));
    assert!(call.variants_truncated());
}

fn both_arm_sql(number: usize, enabled: Option<usize>) -> String {
    let mut sql = format!("SELECT {number}");
    for index in 0..20 {
        let arm = if enabled == Some(index) { "on" } else { "off" };
        sql.push_str(&format!(" /* {arm}{index} */"));
    }
    // The first source guard is !c0, so its false/off assignment is c0=true.
    // A later c0 guard therefore contributes reuse-on in the all-off outcome.
    sql.push_str(if enabled == Some(0) {
        " /* reuse-off */"
    } else {
        " /* reuse-on */"
    });
    sql
}

#[test]
fn both_nonempty_ternary_arms_preserve_all_off_and_each_canonical_singleton() {
    let calls = calls("variants-append-both-arms.ts");
    let call = &calls[0];
    assert_eq!(call.variants.len(), 21);
    assert_eq!(call.variants[0].sql_text, both_arm_sql(20, None));
    for index in 0..20 {
        assert_eq!(
            call.variants[index + 1].sql_text,
            both_arm_sql(20, Some(index))
        );
    }
    assert!(call.variants_truncated());
}

#[test]
fn both_if_else_append_arms_preserve_reversed_and_reused_guard_singletons() {
    let calls = calls("variants-append-both-arms.ts");
    let call = &calls[1];
    assert_eq!(call.variants.len(), 21);
    assert_eq!(call.variants[0].sql_text, both_arm_sql(21, None));
    for index in 0..20 {
        assert_eq!(
            call.variants[index + 1].sql_text,
            both_arm_sql(21, Some(index))
        );
    }
    assert!(call.variants_truncated());
}

#[test]
fn switch_paths_with_shared_sql_do_not_displace_distinct_append_witnesses() {
    let calls = calls("variants-append-switch-witnesses.ts");
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert!(call.variants_truncated());
    assert!(call.is_unanalyzable());
    let texts = texts(call);
    assert!(texts.contains(&"SELECT 31"));
    assert!(texts.contains(&"SELECT 32"));
    for index in 0..20 {
        assert!(
            texts
                .iter()
                .any(|text| *text == format!("SELECT 31 /* g{index} */")
                    || *text == format!("SELECT 32 /* g{index} */")),
            "missing singleton {index}"
        );
    }
}

#[test]
fn reused_guards_exclude_incompatible_suffixes_and_fluent_ids_remain_unique() {
    let calls = calls("variants-append-correlated-heads.ts");
    assert_eq!(calls.len(), 2);
    let mut sql = texts(&calls[0]);
    sql.sort_unstable();
    assert_eq!(
        sql,
        ["SELECT 41 /* off */", "SELECT 41 /* on */ /* same */"]
    );
    assert!(!calls[0].variants_truncated());
    assert_eq!(texts(&calls[1]), ["SELECT 42 /* first */ /* second */"]);
    assert_eq!(calls[1].variants[0].append_sites.len(), 1);
}
