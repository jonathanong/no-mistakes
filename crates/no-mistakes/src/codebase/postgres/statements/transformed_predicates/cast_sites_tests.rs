use super::super::cast_sites::site;
use crate::codebase::postgres::parse::PreparedSql;
use sqlparser::ast::CastKind;
use sqlparser::tokenizer::{Location, Span};

fn source(name: &str) -> String {
    std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-no-transformed-uuid-predicates")
            .join("cast-site-tokens")
            .join(name),
    )
    .unwrap()
}

fn span(start: u64, end: u64) -> Span {
    Span::new(Location::new(1, start), Location::new(1, end))
}

#[test]
fn cast_token_sites_support_qualified_array_types_and_stop_at_context_boundaries() {
    let source = source("valid.sql.tokens");
    let rows = [
        (10, 12, CastKind::TryCast, Some(1)),
        (11, 13, CastKind::SafeCast, Some(1)),
        (6, 10, CastKind::Cast, Some(1)),
        (1, 5, CastKind::DoubleColon, Some(5)),
        (3, 5, CastKind::DoubleColon, Some(7)),
        (9, 11, CastKind::Cast, Some(4)),
        (6, 8, CastKind::Cast, None),
        (16, 18, CastKind::Cast, None),
        (6, 14, CastKind::Cast, Some(1)),
        (6, 8, CastKind::Cast, Some(1)),
    ];
    assert_eq!(source.lines().count(), rows.len());
    for (sql, (start, end, kind, expected)) in source.lines().zip(rows) {
        let prepared = PreparedSql::new(sql);
        assert_eq!(
            site(prepared.tokens(), span(start, end), &kind),
            expected.map(|column| Location::new(1, column)),
            "{sql}"
        );
    }
}

#[test]
fn incomplete_cast_tokens_and_non_cast_function_names_never_invent_identity() {
    let source = source("invalid.sql.tokens");
    let rows = [
        (1, 3, CastKind::DoubleColon),
        (1, 3, CastKind::DoubleColon),
        (1, 3, CastKind::DoubleColon),
        (1, 3, CastKind::DoubleColon),
        (6, 8, CastKind::Cast),
        (11, 13, CastKind::Cast),
        (8, 10, CastKind::Cast),
        (3, 5, CastKind::Cast),
        (6, 24, CastKind::Cast),
    ];
    assert_eq!(source.lines().count(), rows.len());
    for (sql, (start, end, kind)) in source.lines().zip(rows) {
        let prepared = PreparedSql::new(sql);
        assert_eq!(
            site(prepared.tokens(), span(start, end), &kind),
            None,
            "{sql}"
        );
    }
}

#[test]
fn missing_or_misaligned_root_spans_do_not_reuse_nearby_tokens() {
    let source = source("valid.sql.tokens");
    let prepared = PreparedSql::new(source.lines().next().unwrap());
    let tokens = prepared.tokens();
    assert_eq!(site(&tokens[..0], span(10, 12), &CastKind::TryCast), None);
    assert_eq!(site(tokens, span(11, 12), &CastKind::TryCast), None);
    assert_eq!(site(tokens, span(9, 10), &CastKind::TryCast), None);
    assert_eq!(site(tokens, span(10, 12), &CastKind::Cast), None);
}
