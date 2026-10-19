use super::super::{LiteralObjects, Splice};
use super::rewrite;

#[test]
fn a_user_nul_is_not_a_splice_sentinel() {
    let objects = LiteralObjects::default();
    let sql = "SELECT '\0' FROM orders\nOFFSET 1";
    let outcome = rewrite(&objects, sql);
    assert!(!outcome.poisoned);
    assert_eq!(outcome.sql, sql);
}

#[test]
fn a_poisoned_sentinel_keeps_the_surrounding_sql() {
    let mut objects = LiteralObjects::default();
    objects.poisoned.insert("SQL".into());
    objects.literals.borrow_mut().push(Splice {
        name: "SQL".into(),
        literal: "orders".into(),
        fallback: "sql_placeholder_1".into(),
    });
    let outcome = rewrite(&objects, "SELECT id FROM \x000\x00 OFFSET 10");
    assert!(outcome.poisoned);
    assert_eq!(outcome.sql, "SELECT id FROM sql_placeholder_1 OFFSET 10");
    let user_nul = rewrite(&objects, "SELECT '\0' OFFSET 10");
    assert!(!user_nul.poisoned);
    assert_eq!(user_nul.sql, "SELECT '\0' OFFSET 10");
}
