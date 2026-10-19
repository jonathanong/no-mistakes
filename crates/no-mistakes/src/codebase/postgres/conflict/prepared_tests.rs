use super::prepared::{reliable_chunk_tokens, reliable_conflicts, ConflictChunks};
use super::SqlConflictTarget;
use crate::codebase::postgres::parse::{parse_postgres_sql, parse_postgres_sql_lenient};
use crate::codebase::postgres::statements::extract_sql_variant_statement_facts;
use sqlparser::tokenizer::{Span, Token, TokenWithSpan};
use std::sync::Arc;

const FAILED_TARGETLESS: &str =
    "INSERT INTO items (id) VALUES (gen_random_uuid()) ON CONFLICT DO NOTHING RETURNING (";
const COMPLIANT_UPSERT: &str = "INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING";
const FAILED_TARGETED: &str =
    "INSERT INTO items (id) VALUES (gen_random_uuid()) ON CONFLICT (id) DO NOTHING RETURNING (";
const TARGETLESS_MULTI: &str = "INSERT INTO items (id) SELECT input.id FROM unnest(ARRAY[gen_random_uuid(), gen_random_uuid()]) AS input(id) ON CONFLICT DO NOTHING";

fn columns_id(target: &SqlConflictTarget) -> bool {
    matches!(
        target,
        SqlConflictTarget::Columns { expressions, .. }
            if expressions == &["id".to_string()]
    )
}

fn align(earlier: &str, later: &str) -> super::prepared::ConflictRecovery {
    let mut statements = parse_postgres_sql_lenient(earlier);
    let earlier_count = statements.len();
    statements.extend(parse_postgres_sql(later).expect("later statement"));
    let complete = (0..statements.len())
        .map(|index| index >= earlier_count)
        .collect::<Vec<_>>();
    let mut chunks = vec![Some(earlier); earlier_count];
    chunks.extend(std::iter::repeat_n(
        Some(later),
        statements.len() - earlier_count,
    ));
    let orphans: &[&str] = if earlier_count == 0 { &[earlier] } else { &[] };
    reliable_conflicts(&statements, &[], &complete, &chunks, orphans)
}

#[test]
fn failed_targetless_chunk_does_not_attach_to_the_next_upsert() {
    let recovery = align(FAILED_TARGETLESS, COMPLIANT_UPSERT);
    assert!(recovery.failure_overlaps_conflict);
    assert!(
        recovery
            .reliable
            .iter()
            .all(|fact| columns_id(&fact.target)),
        "{:?}",
        recovery.reliable
    );
    assert!(!recovery.reliable.is_empty());
}

#[test]
fn failed_targeted_chunk_keeps_the_later_targetless_insert() {
    let recovery = align(FAILED_TARGETED, TARGETLESS_MULTI);
    assert!(recovery.failure_overlaps_conflict);
    assert!(
        recovery
            .reliable
            .iter()
            .all(|fact| fact.target == SqlConflictTarget::Targetless),
        "{:?}",
        recovery.reliable
    );
    assert_eq!(recovery.reliable.len(), 1);
}

#[test]
fn extracted_failed_sibling_keeps_only_its_own_target() {
    let sql = format!("{FAILED_TARGETLESS}; {COMPLIANT_UPSERT}");
    let facts = extract_sql_variant_statement_facts(&sql, false, &[]);
    assert!(facts.parse_failed);
    let locations = facts.variant_locations.expect("locations");
    assert!(locations.conflict_failure_overlaps);
    assert!(
        locations
            .reliable_conflicts
            .iter()
            .all(|fact| fact.target != SqlConflictTarget::Targetless),
        "{:?}",
        locations.reliable_conflicts
    );
    assert!(
        locations
            .reliable_conflicts
            .iter()
            .any(|fact| columns_id(&fact.target)),
        "{:?}",
        locations.reliable_conflicts
    );
}

fn eof_token() -> TokenWithSpan {
    TokenWithSpan {
        token: Token::EOF,
        span: Span::empty(),
    }
}

/// A missing token chunk is a boundary, not a continuation of the previous text.
/// An incomplete statement there abstains instead of borrowing a sibling target.
#[test]
fn missing_chunk_between_distinct_texts_overlaps_when_incomplete() {
    let statements = parse_postgres_sql("SELECT 1; SELECT 2; SELECT 3").expect("selects");
    assert_eq!(statements.len(), 3);
    let left: Arc<[TokenWithSpan]> = Arc::from([eof_token()]);
    let right: Arc<[TokenWithSpan]> = Arc::from([eof_token(), eof_token()]);
    let chunks = [Some(left), None, Some(right)];
    let recovery = reliable_chunk_tokens(
        &statements,
        &[],
        &ConflictChunks {
            complete: &[true, false, true],
            chunks: &chunks,
            orphans: &[],
        },
    );
    assert!(recovery.failure_overlaps_conflict);
    assert!(recovery.reliable.is_empty());
}

/// A recovered statement with no chunk tokens is not an orphan `ON CONFLICT` chunk.
#[test]
fn statements_without_chunk_tokens_are_not_orphan_chunks() {
    let statement = parse_postgres_sql("SELECT 1")
        .expect("select")
        .pop()
        .expect("one statement");
    let located = crate::codebase::postgres::parse::LocatedStatement {
        statement,
        source: None,
        function_projection: false,
        synthetic_select: false,
        recovered_functions: Vec::new(),
        policy_arbiter: None,
        complete: true,
        chunk_tokens: None,
    };
    assert!(
        crate::codebase::postgres::parse::orphan_chunk_tokens(std::slice::from_ref(&located))
            .is_empty()
    );
}
