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
}
