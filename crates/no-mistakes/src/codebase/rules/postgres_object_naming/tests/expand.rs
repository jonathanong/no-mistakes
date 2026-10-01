use super::super::expand::{expands, middle_matches, suggestion};
use super::super::pattern::{compile_pattern, match_name};

fn expands_to(middle: &str, table: &str, letters: usize) -> bool {
    expands(middle, table, letters)
}

#[test]
fn expansion_examples() {
    let table = "order_line_items";
    for middle in [
        "order_line_items",
        "ord_line_itms",
        "ordr_lin_itms",
        "ORD_LINE_ITMS",
    ] {
        assert!(expands_to(middle, table, 3), "{middle}");
    }
    for middle in [
        "oli",
        "order_items",
        "ord_ln_itms",
        "ord_line_imts",
        "rd_line_items",
    ] {
        assert!(!expands_to(middle, table, 3), "{middle}");
    }
    assert!(expands_to("ord__arch", "order__archives", 3));
    assert!(!expands_to("ord_arch", "order__archives", 3));
    assert!(expands_to("ai_usg_evnts", "ai_usage_events", 3));
    assert!(!expands_to("a_usg_evnts", "ai_usage_events", 3));
    assert!(expands_to("o_l_i", "order_line_items", 1));
    assert!(!expands_to("oli", "order_line_items", 1));
}

#[test]
fn full_name_match_is_ascii_case_insensitive() {
    assert!(middle_matches("ORDERS", "orders", false, 3));
    assert!(middle_matches("Café", "café", false, 3));
    assert!(!middle_matches("CAFÉ", "café", false, 3));
    assert!(!middle_matches("ordrs", "orders", false, 3));
}

#[test]
fn suggestions_follow_the_vowel_hint() {
    assert_eq!(suggestion("orders", 3).as_deref(), Some("ordrs"));
    assert_eq!(suggestion("notes", 3).as_deref(), Some("nts"));
    assert_eq!(
        suggestion("order_line_items", 3).as_deref(),
        Some("ordr_lin_itms")
    );
    assert_eq!(suggestion("ai_events", 3).as_deref(), Some("ai_evnts"));
    assert_eq!(
        suggestion(
            "customer_subscription_renewal_reminder_delivery_attempts",
            3
        )
        .as_deref(),
        Some("cstmr_sbscrptn_rnwl_rmndr_dlvry_attmpts")
    );
    assert_eq!(suggestion("sky", 3), None);
    assert_eq!(
        suggestion("order__archives", 3).as_deref(),
        Some("ordr__archvs")
    );
}

#[test]
fn matching_keeps_the_longest_middle_and_grouped_alternation() {
    let loose = compile_pattern("index", r"^idx_{table}[a-z0-9_]*$").unwrap();
    let matched = match_name(&loose, "idx_orders__x", Some("orders"), true, 3).unwrap();
    assert_eq!(matched.middle, Some((4, 10)));
    let unique = compile_pattern("uniqueIndex", r"^(idx|uq)_{table}__[a-z0-9_]+$").unwrap();
    assert!(match_name(&unique, "uq_orders__id", Some("orders"), false, 3).is_some());
    assert!(match_name(
        &unique,
        "idx_order__archives__id",
        Some("order__archives"),
        false,
        3
    )
    .is_some());
    let trigger = compile_pattern("trigger", r"^trg_{table}__[a-z0-9_]+$").unwrap();
    assert!(match_name(&trigger, "trg_ORDERS__audit", Some("orders"), false, 3).is_some());
}

#[test]
fn plain_patterns_are_not_anchored_implicitly() {
    let pattern = compile_pattern("table", "idx_").unwrap();
    assert!(match_name(&pattern, "xidx_y", None, false, 3).is_some());
    let anchored = compile_pattern("table", "^idx_").unwrap();
    assert!(match_name(&anchored, "xidx_y", None, false, 3).is_none());
}
