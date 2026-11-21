use super::recovered_query;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

#[test]
fn a_table_statement_is_not_a_recovered_query() {
    let statement = Parser::parse_sql(&PostgreSqlDialect {}, "CREATE TABLE orders (id int)")
        .unwrap()
        .remove(0);
    assert!(!recovered_query(&statement));
}
