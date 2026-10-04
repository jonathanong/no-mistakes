use super::*;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use sqlparser::ast::{SetExpr, Statement, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

#[test]
fn explicit_self_alias_does_not_capture_schema_qualified_outer_reference() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-bounded-statements/fixture/sql/explicit-self-alias-lateral.sql",
    );
    let sql = std::fs::read_to_string(path).unwrap();
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, &sql).unwrap();
    let Statement::Query(outer) = &statements[0] else {
        panic!("expected the fixture to contain a query");
    };
    let SetExpr::Select(outer_select) = outer.body.as_ref() else {
        panic!("expected an outer SELECT");
    };
    let TableFactor::Derived {
        subquery: middle, ..
    } = &outer_select.from[0].joins[0].relation
    else {
        panic!("expected a lateral middle query");
    };
    let SetExpr::Select(middle_select) = middle.body.as_ref() else {
        panic!("expected a middle SELECT");
    };
    let TableFactor::Derived {
        subquery: nested, ..
    } = &middle_select.from[0].joins[0].relation
    else {
        panic!("expected a nested lateral query");
    };
    let mut item = SqlBoundItem::new(
        SqlBoundItemKind::Table("public.accounts".to_string()),
        Some("accounts".to_string()),
        (1, 1),
    );
    item.alias_explicit = true;
    let resolver = Resolver::new(&[item], BTreeMap::new(), None);

    assert!(!resolver.reads(nested).certain);
}
