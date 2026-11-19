use super::*;

#[test]
fn finite_string_guards_keep_only_feasible_sql_variants() {
    let calls = calls("variants-correlated-finite-guards.ts");
    assert_eq!(calls.len(), 13);

    let mut list_either = texts(&calls[0]);
    list_either.sort_unstable();
    assert_eq!(
        list_either,
        [
            "/* listEither */ WITH candidates AS (SELECT id FROM gadgets) SELECT id FROM candidates LIMIT 5",
            "/* listEither */ WITH candidates AS (SELECT id FROM widgets UNION ALL SELECT id FROM gadgets) SELECT id FROM candidates LIMIT 5",
            "/* listEither */ WITH candidates AS (SELECT id FROM widgets) SELECT id FROM candidates LIMIT 5",
        ]
    );
    assert!(!calls[0].variants_truncated());

    let mut nested = texts(&calls[3]);
    nested.sort_unstable();
    assert_eq!(
        nested,
        [
            "SELECT 1 /* gadgets */",
            "SELECT 1 /* gadgets */ /* all */",
            "SELECT 1 /* widgets */",
        ]
    );
    assert!(!calls[3].variants_truncated());
}

#[test]
fn independent_and_unproven_guards_retain_each_sql_combination() {
    let calls = calls("variants-correlated-finite-guards.ts");
    assert_eq!(calls.len(), 13);
    for (index, description) in [
        (1, "distinct parameters"),
        (2, "reassigned parameter"),
        (4, "open string type"),
        (5, "numeric type"),
    ] {
        let mut variants = texts(&calls[index]);
        variants.sort_unstable();
        assert_eq!(
            variants,
            [
                "SELECT 1",
                "SELECT 1 /* gadgets */",
                "SELECT 1 /* widgets */",
                "SELECT 1 /* widgets */ /* gadgets */",
            ],
            "{description}"
        );
        assert!(!calls[index].variants_truncated(), "{description}");
    }
}

#[test]
fn unsupported_operands_and_exhausted_predicate_budgets_keep_both_arms() {
    let calls = calls("variants-correlated-finite-guards.ts");
    assert_eq!(calls.len(), 13);
    for (index, description) in [
        (7, "object operand"),
        (8, "array operand"),
        (9, "single predicate pair budget"),
        (10, "combined parent path budget"),
        (11, "predicate depth budget"),
    ] {
        let mut variants = texts(&calls[index]);
        variants.sort_unstable();
        assert_eq!(
            variants,
            ["SELECT 2", "SELECT 2 /* branch */"],
            "{description}"
        );
        assert!(!calls[index].variants_truncated(), "{description}");
    }
}

#[test]
fn independent_opaque_tail_keeps_both_outcomes_within_each_finite_mode() {
    let calls = calls("variants-correlated-finite-guards.ts");
    let mut variants = texts(&calls[6]);
    variants.sort_unstable();
    assert_eq!(
        variants,
        [
            "SELECT 1",
            "SELECT 1 /* gadgets */",
            "SELECT 1 /* widgets */"
        ]
    );
    assert!(!calls[6].variants_truncated());
}

#[test]
fn branch_reassignment_keeps_every_feasible_variant_after_the_join() {
    let calls = calls("variants-correlated-finite-guards.ts");
    let mut variants = texts(&calls[12]);
    variants.sort_unstable();
    assert_eq!(
        variants,
        [
            "/* reassignedInsideBranch */ WITH candidates AS () SELECT id FROM candidates LIMIT 5",
            "/* reassignedInsideBranch */ WITH candidates AS (SELECT id FROM items) SELECT id FROM candidates LIMIT 5",
            "/* reassignedInsideBranch */ WITH candidates AS (SELECT id FROM widgets) SELECT id FROM candidates LIMIT 5",
            "/* reassignedInsideBranch */ WITH candidates AS (SELECT id FROM widgetsSELECT id FROM items) SELECT id FROM candidates LIMIT 5",
        ]
    );
    assert!(!calls[12].variants_truncated());
}
