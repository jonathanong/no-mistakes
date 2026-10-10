use super::procedural_occurrences::{block, kinds};
use crate::codebase::postgres::parse::prepare_postgres_tokens;

#[test]
fn ordinary_sql_leaves_occurrence_tokens_empty() {
    // Ordinary SQL must not keep a second token inventory.
    let prepared = prepare_postgres_tokens("SELECT 1");
    assert!(!prepared.tokens.is_empty());
    assert!(prepared.occurrence_tokens.is_empty());
}

#[test]
fn multi_relation_row_lock_stays_one_dml_occurrence() {
    // Parser prep splits `OF a, b` into two locking clauses. Classification
    // still sees the original `UPDATE` once.
    let (_, parsed) = block(
        "DO $$ BEGIN FOR r IN SELECT * FROM a, b FOR UPDATE OF a, b LOOP NULL; END LOOP; END $$;",
    );
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Dml\", \"Unknown\"]"]
    );
}
