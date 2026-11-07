use super::super::{query, Scope};
use crate::codebase::postgres::{parse_postgres_sql, SqlBoundItemKind};
use sqlparser::ast::Statement;

#[test]
fn lateral_limits_follow_caller_array_columns_and_relation_scope() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/issue-3100-limits.sql"
    ));
    let statements = parse_postgres_sql(sql).unwrap();
    let expected = [
        true, false, true, true, true, true, false, false, false, false, false, false, false, true,
        false, false, false, false, false, false, true, false, false, false, false, false, false,
        true, false, false,
    ];
    assert_eq!(statements.len(), expected.len());
    for (index, (statement, expected)) in statements.iter().zip(expected).enumerate() {
        let Statement::Query(ast) = statement else {
            panic!("saved fixture must contain queries");
        };
        let bound = query::bound_query(ast, &Scope::default(), None);
        let tested = bound
            .items
            .iter()
            .rev()
            .find_map(|item| match &item.kind {
                SqlBoundItemKind::Query(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or(&bound);
        assert_eq!(tested.capped, expected, "fixture SQL line {}", index + 1);
    }
}

#[test]
fn visible_limit_relations_are_built_only_for_real_identifiers() {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("sql/lazy-lateral-limits.sql");
    let sql = std::fs::read_to_string(path).unwrap();
    let statements = parse_postgres_sql(&sql).unwrap();
    let expected = [
        (false, 0),
        (true, 0),
        (true, 0),
        (true, 0),
        (true, 0),
        (false, 0),
        (false, 1),
        (false, 1),
        (true, 1),
        (false, 1),
        (false, 1),
    ];
    assert_eq!(statements.len(), expected.len());
    for (index, (statement, (capped, builds))) in statements.iter().zip(expected).enumerate() {
        let Statement::Query(ast) = statement else {
            panic!("saved fixture must contain queries");
        };
        super::super::limit_columns::work::begin();
        let bound = query::bound_query(ast, &Scope::default(), None);
        let observed = super::super::limit_columns::work::finish();
        let tested = bound
            .items
            .iter()
            .rev()
            .find_map(|item| match &item.kind {
                SqlBoundItemKind::Query(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or(&bound);
        assert_eq!(tested.capped, capped, "fixture SQL line {}", index + 1);
        assert_eq!(observed, builds, "fixture SQL line {}", index + 1);
    }
}

#[test]
fn builder_relations_materialize_only_for_lateral_derived_queries() {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("sql/lazy-builder-limits.sql");
    let sql = std::fs::read_to_string(path).unwrap();
    let statements = parse_postgres_sql(&sql).unwrap();
    let expected = [
        (0, false, (0, 0)),
        (1, true, (1, 2)),
        (2, true, (1, 3)),
        (3, false, (1, 1)),
        (4, false, (1, 3)),
        (5, false, (1, 4)),
        (10, false, (1, 4)),
        (11, true, (0, 0)),
    ];
    assert_eq!(statements.len(), 12);
    for (index, capped, work) in expected {
        let statement = &statements[index];
        let Statement::Query(ast) = statement else {
            panic!("saved fixture must contain queries");
        };
        super::super::limit_columns::work::begin_builder();
        let bound = query::bound_query(ast, &Scope::default(), None);
        let observed = super::super::limit_columns::work::finish_builder();
        let tested = bound
            .items
            .iter()
            .rev()
            .find_map(|item| match &item.kind {
                SqlBoundItemKind::Query(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or(&bound);
        assert_eq!(tested.capped, capped, "fixture SQL line {}", index + 1);
        assert_eq!(observed, work, "fixture SQL line {}", index + 1);
    }

    let Statement::Query(seed) = &statements[6] else {
        panic!("saved fixture must contain a query");
    };
    let Statement::Query(inherited) = &statements[7] else {
        panic!("saved fixture must contain a query");
    };
    let sqlparser::ast::SetExpr::Select(select) = seed.body.as_ref() else {
        panic!("saved fixture must contain a select");
    };
    let mut scope = Scope::default();
    scope.limit_columns.add(&select.from[0].relation, None);
    super::super::limit_columns::work::begin_builder();
    let bound = query::bound_query(inherited, &scope, None);
    let observed = super::super::limit_columns::work::finish_builder();
    let SqlBoundItemKind::Query(inner) = &bound.items[0].kind else {
        panic!("saved fixture must contain a derived query");
    };
    assert!(inner.capped);
    assert_eq!(observed, (1, 1));

    let Statement::Query(split) = &statements[8] else {
        panic!("saved fixture must contain a query");
    };
    let sqlparser::ast::SetExpr::Select(select) = split.body.as_ref() else {
        panic!("saved fixture must contain a select");
    };
    let empty_scope = Scope::default();
    let mut builder = super::super::items::Builder::new(&empty_scope, None);
    super::super::limit_columns::work::begin_builder();
    builder.target_tables(&select.from[..1]);
    builder.tables(&select.from[1..]);
    let items = builder.finish(None);
    let observed = super::super::limit_columns::work::finish_builder();
    let SqlBoundItemKind::Query(inner) = &items[2].kind else {
        panic!("saved fixture must contain a derived query");
    };
    assert!(inner.capped);
    assert_eq!(observed, (1, 3));

    let Statement::Query(nested) = &statements[9] else {
        panic!("saved fixture must contain a query");
    };
    super::super::limit_columns::work::begin_builder();
    let bound = query::bound_query(nested, &Scope::default(), None);
    let observed = super::super::limit_columns::work::finish_builder();
    let SqlBoundItemKind::Query(middle) = &bound.items[1].kind else {
        panic!("saved fixture must contain a derived query");
    };
    let SqlBoundItemKind::Query(inner) = &middle.items[0].kind else {
        panic!("saved fixture must contain a nested derived query");
    };
    assert!(inner.capped);
    assert_eq!(observed, (2, 3));
}
