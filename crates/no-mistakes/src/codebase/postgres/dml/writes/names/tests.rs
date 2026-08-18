use super::{table_factor_name, table_object_name};
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{Statement, TableFactor, TableObject};

#[test]
fn unnamed_table_objects_and_factors_are_skipped() {
    let Statement::Query(query) = parse_postgres_sql("SELECT 1")
        .unwrap()
        .pop()
        .expect("select")
    else {
        panic!("expected query");
    };
    assert_eq!(
        table_object_name(&TableObject::TableQuery(query.clone())),
        None
    );
    assert_eq!(
        table_factor_name(&TableFactor::Derived {
            lateral: false,
            subquery: query,
            alias: None,
            sample: None,
        }),
        None
    );
}
