use super::*;
use crate::codebase::postgres::{canonical_bound_expr, parse_postgres_expression};

#[test]
fn civil_days_cover_epoch_and_leap_years() {
    assert_eq!(days_from_civil(1970, 1, 1), Some(0));
    let year = |year, month, day| days_from_civil(year, month, day).unwrap();
    assert_eq!(year(2026, 12, 31) - year(2026, 1, 1) + 1, 365);
    assert_eq!(year(2024, 12, 31) - year(2024, 1, 1) + 1, 366);
    assert_eq!(year(2026, 2, 9) - year(2026, 1, 1) + 1, 40);
    assert_eq!(year(2034, 3, 19) - year(2026, 1, 1) + 1, 3000);
    assert_eq!(year(2043, 3, 19) - year(2035, 1, 1) + 1, 3000);
    assert_eq!(days_from_civil(2026, 2, 29), None);
    assert!(days_from_civil(2024, 2, 29).is_some());
}

#[test]
fn date_predicate_counts_inclusive_days() {
    let expr =
        parse_postgres_expression("day >= DATE '2026-01-01' AND day <= DATE '2026-12-31'").unwrap();
    let expr = canonical_bound_expr(&expr);
    let mut conjuncts = Vec::new();
    dnf::and_conjuncts(&expr, &mut conjuncts);
    assert_eq!(
        closed_span("day", &conjuncts, Literal::Date)
            .unwrap()
            .width(),
        Some(365)
    );
}

#[test]
fn checks_accept_between_and_paired_bounds() {
    assert_eq!(
        check_span("CHECK (slot BETWEEN 1 AND 50)", "slot")
            .unwrap()
            .width(),
        Some(50)
    );
    assert_eq!(
        check_span("CHECK ((rank >= 1) AND (rank <= 10))", "rank")
            .unwrap()
            .width(),
        Some(10)
    );
    assert!(check_span("CHECK (slot BETWEEN 1 AND 50 OR slot = 1)", "slot").is_none());
    assert!(check_span("CHECK (slot NOT BETWEEN 1 AND 50)", "slot").is_none());
    assert!(check_span("UNIQUE (slot)", "slot").is_none());
    assert!(check_span("CHECK (slot BETWEEN 1 AND 50) NOT ENFORCED", "slot").is_none());
    assert!(check_span("CHECK (slot >= 5 AND slot <= 3)", "slot").is_none());
}

#[test]
fn comparisons_cover_both_orders_casts_and_negative_bounds() {
    let width = |sql: &str, column: &str, kind: Literal| {
        let expr = parse_postgres_expression(sql).unwrap();
        let mut conjuncts = Vec::new();
        dnf::and_conjuncts(&expr, &mut conjuncts);
        closed_span(column, &conjuncts, kind).and_then(|span| span.width())
    };
    assert_eq!(
        width("slot BETWEEN 1 AND 50", "slot", Literal::Integer),
        Some(50)
    );
    assert_eq!(
        width("1 <= slot AND slot <= 10", "slot", Literal::Integer),
        Some(10)
    );
    assert_eq!(
        width("slot >= -3 AND slot <= -1", "slot", Literal::Integer),
        Some(3)
    );
    assert_eq!(
        width(
            "CAST(slot AS integer) >= 1 AND CAST(slot AS integer) <= 4",
            "slot",
            Literal::Integer
        ),
        Some(4)
    );
    assert_eq!(
        width(
            "slot >= CAST(1 AS integer) AND slot <= CAST('4' AS integer)",
            "slot",
            Literal::Integer
        ),
        Some(4)
    );
    assert_eq!(width("slot >= other", "slot", Literal::Integer), None);
    assert_eq!(
        width(
            "CAST(slot AS text) >= 1 AND CAST(slot AS text) <= 4",
            "slot",
            Literal::Integer
        ),
        None
    );
    assert_eq!(
        width("slot >= 1.5 AND slot <= 4", "slot", Literal::Integer),
        None
    );
    assert_eq!(
        width(
            "day >= CAST('2026-01-01' AS date) AND day <= CAST('2026-01-02' AS date)",
            "day",
            Literal::Date
        ),
        Some(2)
    );
    assert!(days_from_civil(2026, 4, 30).is_some());
    assert_eq!(days_from_civil(0, 1, 1), None);
    assert_eq!(days_from_civil(2026, 13, 1), None);
    assert_eq!(days_from_civil(2026, 4, 31), None);
    let low = closed_span(
        "slot",
        &[parse_postgres_expression("slot BETWEEN 1 AND 2").unwrap()],
        Literal::Integer,
    )
    .unwrap();
    let high = closed_span(
        "slot",
        &[parse_postgres_expression("slot BETWEEN 5 AND 6").unwrap()],
        Literal::Integer,
    )
    .unwrap();
    assert!(intersect(Some(low), Some(high)).is_none());
    assert!(intersect(None, None).is_none());
    assert_eq!(intersect(Some(low), None).unwrap().width(), low.width());
}

#[test]
fn or_bounds_that_do_not_match_the_pin_arms_share_only_common_conjuncts() {
    let mismatched = parse_postgres_expression(
        "day >= DATE '2026-01-01' AND day <= DATE '2026-01-02' OR day >= DATE '2026-03-01' AND day <= DATE '2026-03-02'",
    )
    .unwrap();
    let shared = query_spans(std::slice::from_ref(&mismatched), 1);
    assert_eq!(shared.len(), 1);
    assert!(shared[0].is_empty());
    let mut arms = String::from("c = 1");
    for value in 2..=40 {
        arms.push_str(&format!(" OR c = {value}"));
    }
    let wide = parse_postgres_expression(&arms).unwrap();
    assert_eq!(query_spans(std::slice::from_ref(&wide), 1).len(), 1);
    let mut factors = Vec::new();
    for start in [1, 4, 7, 10] {
        factors.push(format!(
            "(c = {start} OR c = {} OR c = {})",
            start + 1,
            start + 2
        ));
    }
    let product = parse_postgres_expression(&factors.join(" AND ")).unwrap();
    assert!(query_spans(std::slice::from_ref(&product), 4)
        .iter()
        .all(|arm| arm.is_empty()));
}
