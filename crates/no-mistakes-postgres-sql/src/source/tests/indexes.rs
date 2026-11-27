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

#[test]
fn create_index_on_only_preserves_typed_facts_and_recovers_malformed_neighbors() {
    let basic = super::facts("create-index-only-basic.sql");
    assert!(basic.diagnostics.is_empty(), "{:?}", basic.diagnostics);
    assert_eq!(basic.statements.len(), 1);
    let PostgresSqlStatementKind::CreateIndex { index: basic_index } = &basic.statements[0].facts
    else {
        panic!()
    };
    assert!(basic_index.only);
    assert_eq!(basic_index.table.parts[0].identity, "example");
    assert_eq!(basic_index.keys[0].expression.sql, "id");

    let source = super::fixture("create-index-only.sql");
    let facts = super::facts("create-index-only.sql");
    assert_eq!(facts.diagnostics.len(), 4, "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 9);
    assert_eq!(facts.statements[0].ordinal, 0);
    assert_eq!(
        facts.statements[0].sql,
        "CREATE INDEX example_idx ON ONLY example (id);"
    );
    assert_eq!(facts.statements[1].ordinal, 1);
    assert_eq!(facts.statements[2].ordinal, 3);
    assert_eq!(facts.statements[3].ordinal, 4);
    assert_eq!(facts.statements[4].ordinal, 5);
    assert_eq!(facts.statements[5].ordinal, 7);
    assert!(matches!(
        facts.statements[5].facts,
        PostgresSqlStatementKind::Select { .. }
    ));
    assert_eq!(facts.statements[6].ordinal, 8);
    let PostgresSqlStatementKind::CreateView { view } = &facts.statements[6].facts else {
        panic!()
    };
    assert!(view.query.contains("ON ONLY = b.id"));
    assert_eq!(facts.statements[7].ordinal, 9);
    let PostgresSqlStatementKind::Wrapper { wrapper } = &facts.statements[7].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::CreateIndex { index } = &wrapper.statements[0].facts else {
        panic!()
    };
    assert!(index.only);
    for statement in &facts.statements {
        assert_eq!(
            &source[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
    }

    let PostgresSqlStatementKind::CreateIndex { index } = &facts.statements[0].facts else {
        panic!()
    };
    assert_eq!(index.table.parts[0].identity, "example");
    assert!(index.only);
    assert_eq!(index.keys[0].expression.sql, "id");
    let identity: serde_json::Value = serde_json::from_str(&index.structural_identity).unwrap();
    assert_eq!(identity["only"], true);

    let PostgresSqlStatementKind::CreateIndex { index } = &facts.statements[1].facts else {
        panic!()
    };
    assert!(index.unique);
    assert!(index.only);

    let PostgresSqlStatementKind::CreateIndex { index: plain_index } = &facts.statements[8].facts
    else {
        panic!()
    };
    assert!(!plain_index.only);
    assert_eq!(
        plain_index.structural_identity,
        r#"{"table":[["example",false]],"method":"btree","unique":false,"nullsDistinct":true,"keys":[["{\"Identifier\":{\"value\":\"id\",\"quote_style\":null}}",true,false,null]],"include":[],"predicate":null,"options":[]}"#
    );
    let plain_identity: serde_json::Value =
        serde_json::from_str(&plain_index.structural_identity).unwrap();
    let mut only_identity = identity;
    assert_eq!(
        only_identity.as_object_mut().unwrap().remove("only"),
        Some(true.into())
    );
    assert_eq!(only_identity, plain_identity);
    assert_eq!(index.table.parts[0].identity, "app");
    assert_eq!(index.table.parts[1].identity, "Accounts");
    assert!(index.table.parts[1].quoted);
    assert_eq!(index.method, "gin");
    assert_eq!(index.predicate.as_ref().unwrap().sql, "active");

    let PostgresSqlStatementKind::CreateIndex { index } = &facts.statements[2].facts else {
        panic!()
    };
    assert!(!index.only);
    assert!(index.table.parts[0].quoted);
    assert_eq!(index.table.parts[0].identity, "ONLY");

    let PostgresSqlStatementKind::CreateIndex { index } = &facts.statements[3].facts else {
        panic!()
    };
    assert!(!index.only);
    assert_eq!(index.keys[0].expression.sql, "ONLY");

    let PostgresSqlStatementKind::CreateIndex { index } = &facts.statements[4].facts else {
        panic!()
    };
    assert!(!index.only);
    assert_eq!(index.predicate.as_ref().unwrap().sql, "ONLY IS TRUE");
}

#[test]
fn truncated_create_index_headers_stay_diagnostic() {
    for name in [
        "create-index-only-truncated-create.sql",
        "create-index-only-truncated-unique.sql",
        "create-index-only-truncated-on.sql",
        "create-index-only-truncated-index.sql",
    ] {
        let facts = super::facts(name);
        assert!(
            facts.statements.is_empty(),
            "{name}: {:?}",
            facts.statements
        );
        assert!(!facts.diagnostics.is_empty(), "{name}");
    }
}
