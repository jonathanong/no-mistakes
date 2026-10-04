use super::names;
use crate::codebase::postgres::parse::parse_postgres_sql;
use sqlparser::ast::Statement;

fn view_queries(sql: &str) -> Vec<sqlparser::ast::Query> {
    parse_postgres_sql(sql)
        .unwrap()
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::CreateView(view) => Some(*view.query),
            _ => None,
        })
        .collect()
}

#[test]
fn collects_scalar_view_relations_and_skips_cte_aliases() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-all-dependencies.sql"
    ));
    let queries = view_queries(sql);
    assert_eq!(queries.len(), 3);
    assert_eq!(names(&queries[0]), ["helper".to_string()].into());
    assert_eq!(
        names(&queries[1]),
        [
            "public.accounts".to_string(),
            "public.order_lines".to_string()
        ]
        .into()
    );
    assert!(names(&queries[2]).is_empty());
}

#[test]
fn nonrecursive_ctes_do_not_hide_physical_self_or_forward_references() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-view-cte-visibility.sql"
    ));
    let queries = view_queries(sql);
    assert_eq!(queries.len(), 4);
    assert_eq!(names(&queries[0]), ["helper".to_string()].into());
    assert!(names(&queries[1]).is_empty());
    assert_eq!(names(&queries[2]), ["future".to_string()].into());
    assert!(names(&queries[3]).is_empty());
}
