use super::super::*;

#[test]
fn expression_references_and_identity_borrow_the_same_source_ast() {
    let facts = facts("expressions.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::CreateIndex { index } = &facts.statements[0].facts else {
        panic!()
    };
    assert!(index.keys[0]
        .expression
        .columns
        .iter()
        .any(|column| column.parts.len() == 2));
}
use super::facts;

#[test]
fn index_identity_preserves_semantics_and_normalizes_sort_defaults() {
    let facts = facts("indexes.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let indexes = facts
        .statements
        .iter()
        .map(|statement| match &statement.facts {
            PostgresSqlStatementKind::CreateIndex { index } => index,
            _ => panic!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        indexes[0].structural_identity,
        indexes[1].structural_identity
    );
    assert_eq!(
        indexes[2].structural_identity,
        indexes[3].structural_identity
    );
    assert_ne!(
        indexes[0].structural_identity,
        indexes[2].structural_identity
    );
    assert_ne!(
        indexes[0].structural_identity,
        indexes[5].structural_identity
    );
    assert_ne!(
        indexes[6].structural_identity,
        indexes[7].structural_identity
    );
    assert_ne!(
        indexes[8].structural_identity,
        indexes[9].structural_identity
    );
    assert_eq!(
        indexes[0].structural_identity,
        indexes[10].structural_identity
    );
    assert!(indexes[4].unique);
    assert!(!indexes[4].nulls_distinct);
    assert_eq!(indexes[4].include[0].identity, "id");
    assert_eq!(
        indexes[4].keys[0].operator_class.as_ref().unwrap().parts[0].identity,
        "text_pattern_ops"
    );
    assert!(indexes[4].predicate.is_some());
}

#[test]
fn index_methods_preserve_quoted_case_and_fold_unquoted_case() {
    let facts = facts("index-methods.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let indexes = facts
        .statements
        .iter()
        .map(|statement| {
            let PostgresSqlStatementKind::CreateIndex { index } = &statement.facts else {
                panic!()
            };
            index
        })
        .collect::<Vec<_>>();
    assert_eq!(indexes[0].method, "\"CustomAM\"");
    assert_eq!(indexes[1].method, "\"CUSTOMAM\"");
    assert_ne!(
        indexes[0].structural_identity,
        indexes[1].structural_identity
    );
    assert_eq!(indexes[2].method, "customam");
    assert_eq!(
        indexes[2].structural_identity,
        indexes[3].structural_identity
    );
}
