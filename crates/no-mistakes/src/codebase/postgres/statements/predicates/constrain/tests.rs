use super::refs::qualifier_and_column;
use sqlparser::ast::Ident;

#[test]
fn a_single_part_is_not_a_qualified_column() {
    assert!(qualifier_and_column(&[Ident::new("account_id")]).is_none());
}
