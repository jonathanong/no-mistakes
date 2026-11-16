use super::*;
use crate::codebase::postgres::{canonical_bound_expr, parse_postgres_expression};

fn seats() -> Vec<String> {
    vec!["seats".to_string(), "public.seats".to_string()]
}

fn width(sql: &str, names: &[String]) -> Option<u64> {
    owned_check(sql, "slot", names).and_then(|span| span.width())
}

#[test]
fn qualified_checks_match_only_this_table() {
    let names = seats();
    assert_eq!(width("CHECK (slot BETWEEN 1 AND 5)", &names), Some(5));
    assert_eq!(width("CHECK (seats.slot BETWEEN 1 AND 5)", &names), Some(5));
    assert_eq!(width("CHECK (Seats.slot BETWEEN 1 AND 5)", &names), Some(5));
    assert_eq!(
        width("CHECK (public.seats.slot BETWEEN 1 AND 5)", &names),
        Some(5)
    );
    assert_eq!(
        width(
            "CHECK ((public.seats.slot >= 1) AND (public.seats.slot <= 5))",
            &names
        ),
        Some(5)
    );
    assert_eq!(
        width(
            "CHECK (CAST(seats.slot AS integer) BETWEEN 1 AND 4)",
            &names
        ),
        Some(4)
    );
    assert_eq!(
        width("CHECK (seats.slot::integer BETWEEN 1 AND 4)", &names),
        Some(4)
    );
    assert_eq!(width("CHECK (other.slot BETWEEN 1 AND 5)", &names), None);
    assert_eq!(width("CHECK (seats.other BETWEEN 1 AND 5)", &names), None);
    assert_eq!(
        width("CHECK (audit.seats.slot BETWEEN 1 AND 5)", &names),
        None
    );
    assert_eq!(
        width(
            "CHECK (public.seats.slot BETWEEN 1 AND 5)",
            &["seats".to_string()]
        ),
        None
    );
    assert_eq!(
        width(
            "CHECK (seats.slot NOT BETWEEN 1 AND 9 AND seats.slot >= 1 AND seats.slot <= 5)",
            &names
        ),
        None
    );
    let expr = parse_postgres_expression("seats.slot BETWEEN 1 AND 5").unwrap();
    let expr = canonical_bound_expr(&expr);
    let mut conjuncts = Vec::new();
    dnf::and_conjuncts(&expr, &mut conjuncts);
    assert!(closed_span("slot", &conjuncts, Literal::Integer).is_none());
}
