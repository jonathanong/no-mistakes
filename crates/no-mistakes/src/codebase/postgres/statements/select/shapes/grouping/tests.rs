use super::{empty_grouping_set_multiplicity, has_group_by};
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{Expr, GroupByExpr, GroupByWithModifier, Ident, Select, SetExpr, Statement};

fn parsed_select() -> Select {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/shape-grouping-helper.sql");
    let sql = std::fs::read_to_string(path).unwrap();
    let Statement::Query(query) = parse_postgres_sql(&sql).unwrap().remove(0) else {
        panic!("expected query");
    };
    let SetExpr::Select(select) = *query.body else {
        panic!("expected SELECT");
    };
    *select
}

#[test]
fn grouping_variants_and_empty_alternatives_keep_their_cardinality() {
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::GroupingSets(Vec::new()), false),
        Some(1)
    );
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::Rollup(Vec::new()), false),
        Some(1)
    );
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::Cube(Vec::new()), false),
        Some(1)
    );
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::Cube(vec![Vec::new()]), false),
        Some(2)
    );
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::Rollup(vec![Vec::new()]), false),
        Some(2)
    );
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::Rollup(vec![Vec::new()]), true),
        Some(1)
    );
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::Cube(vec![Vec::new()]), true),
        Some(1)
    );
    assert_eq!(
        empty_grouping_set_multiplicity(&Expr::GroupingSets(vec![Vec::new(), Vec::new()]), true),
        Some(1)
    );
}

#[test]
fn unsupported_grouping_ast_variants_are_conservatively_grouped() {
    let mut select = parsed_select();
    select.group_by = GroupByExpr::All(Vec::new());
    assert!(has_group_by(&select));

    select.group_by = GroupByExpr::Expressions(Vec::new(), vec![GroupByWithModifier::Rollup]);
    assert!(has_group_by(&select));
}

#[test]
fn quoted_distinct_identifier_is_not_the_postgres_modifier() {
    let mut select = parsed_select();
    select.group_by = GroupByExpr::Expressions(
        vec![Expr::Identifier(Ident::with_quote('"', "distinct"))],
        vec![GroupByWithModifier::GroupingSets(Expr::GroupingSets(vec![
            Vec::new(),
            Vec::new(),
        ]))],
    );
    assert!(has_group_by(&select));

    select.group_by = GroupByExpr::Expressions(
        vec![
            Expr::Identifier(Ident::new("distinct")),
            Expr::Identifier(Ident::new("account_id")),
        ],
        Vec::new(),
    );
    assert!(has_group_by(&select));
}
