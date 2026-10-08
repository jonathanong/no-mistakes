use super::*;
use crate::codebase::postgres::source::tests::fixture;
use sqlparser::{ast::*, dialect::PostgreSqlDialect, parser::Parser};

fn saved_insert(index: usize) -> (String, Insert) {
    let sql = fixture("insert-column-sources.sql");
    let Statement::Insert(insert) = Parser::parse_sql(&PostgreSqlDialect {}, &sql)
        .unwrap()
        .remove(index)
    else {
        panic!()
    };
    (sql, insert)
}
fn reason(value: PostgresSqlInsertColumnSources) -> String {
    serde_json::to_value(value).unwrap()["reason"]
        .as_str()
        .unwrap()
        .to_owned()
}
#[test]
fn column_sources_defensive_prepared_input_boundaries() {
    let (sql, insert) = saved_insert(0);
    let locations = Locations::new(&sql);
    let missing = PostgresSqlInsertSource::DefaultValues;
    assert_eq!(
        reason(project(&insert.columns, None, &missing, &[], &locations)),
        "defaultValues"
    );
    assert_eq!(
        reason(project(
            &insert.columns,
            insert.source.as_deref(),
            &missing,
            &[],
            &locations
        )),
        "unsupportedSource"
    );
    let empty = PostgresSqlInsertSource::Values {
        rows: Vec::new(),
        span: None,
    };
    assert_eq!(
        reason(project(
            &insert.columns,
            insert.source.as_deref(),
            &empty,
            &[],
            &locations
        )),
        "unsupportedSource"
    );
    let missing_column = PostgresSqlInsertSource::Values {
        rows: vec![Vec::new()],
        span: None,
    };
    assert_eq!(
        reason(project(
            &insert.columns,
            insert.source.as_deref(),
            &missing_column,
            &[],
            &locations
        )),
        "unsupportedSource"
    );
    let mut query = *insert.source.unwrap();
    let SetExpr::Values(values) = query.body.as_mut() else {
        panic!()
    };
    values.rows.clear();
    assert_eq!(
        reason(project(
            &insert.columns,
            Some(&query),
            &missing,
            &[],
            &locations
        )),
        "emptySource"
    );
}
#[test]
fn column_sources_collect_defensive_ast_boundaries() {
    let (_, insert) = saved_insert(3);
    let mut query = *insert.source.unwrap();
    let mut branches = Vec::new();
    assert_eq!(
        reason(
            collect(&query.body, vec![1, 0], 64, &mut branches)
                .err()
                .unwrap()
        ),
        "nestingLimit"
    );
    assert!(branches.is_empty());
    drop(branches);
    let SetExpr::Select(select) = query.body.as_mut() else {
        panic!()
    };
    let expr = match &select.projection[0] {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => expr.clone(),
        _ => panic!(),
    };
    select.projection = vec![SelectItem::ExprWithAliases {
        expr,
        aliases: vec![Ident::new("saved_alias")],
    }];
    assert_eq!(
        reason(
            collect(&query.body, vec![1], 0, &mut Vec::new())
                .err()
                .unwrap()
        ),
        "aliasExpansion"
    );
    let SetExpr::Select(select) = query.body.as_mut() else {
        panic!()
    };
    select.projection.clear();
    assert_eq!(
        reason(
            collect(&query.body, Vec::new(), 0, &mut Vec::new())
                .err()
                .unwrap()
        ),
        "emptySource"
    );
    // A restricted-query AST can contain DML in other dialects; PostgreSQL projection must fail closed.
    let (_, saved) = saved_insert(0);
    let unsupported = SetExpr::Insert(Statement::Insert(saved));
    assert_eq!(
        reason(
            collect(&unsupported, Vec::new(), 0, &mut Vec::new())
                .err()
                .unwrap()
        ),
        "unsupportedSource"
    );
    assert!(validate_arity(2, &[]).is_none());
}

#[test]
fn column_sources_propagate_unsupported_nested_branch_errors() {
    let (_, saved) = saved_insert(0);
    let unsupported = SetExpr::Insert(Statement::Insert(saved));
    let (_, select_insert) = saved_insert(3);
    let mut nested_query = *select_insert.source.unwrap();
    nested_query.body = Box::new(unsupported.clone());
    // Recursive collection must preserve the unsupported result rather than report a partial map.
    let nested = SetExpr::Query(Box::new(nested_query));
    assert_eq!(
        reason(
            collect(&nested, Vec::new(), 0, &mut Vec::new())
                .err()
                .unwrap()
        ),
        "unsupportedSource"
    );
    for unsupported_left in [true, false] {
        let (_, set_insert) = saved_insert(1);
        let mut body = set_insert.source.unwrap().body;
        let SetExpr::SetOperation { left, right, .. } = body.as_mut() else {
            panic!()
        };
        if unsupported_left {
            **left = unsupported.clone();
        } else {
            **right = unsupported.clone();
        }
        assert_eq!(
            reason(
                collect(&body, Vec::new(), 0, &mut Vec::new())
                    .err()
                    .unwrap()
            ),
            "unsupportedSource"
        );
    }
}
