use sqlparser::ast::{Expr, Function, Visit, Visitor};
use sqlparser::dialect::{BigQueryDialect, ClickHouseDialect, Dialect, PostgreSqlDialect};
use std::ops::ControlFlow;

#[test]
fn parser_array_type_variants_keep_custom_conversion_inputs_opaque() {
    struct First(Option<Function>);
    impl Visitor for First {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if let Expr::Function(function) = expr {
                self.0 = Some(function.clone());
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
    }
    // The typed input helper accepts parser ASTs; dialect adapters represent array
    // element types differently. None may hide a custom input/cast function.
    for (dialect, sql) in [
        (&PostgreSqlDialect {} as &dyn Dialect, include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/conditional-input-type-postgres.sql"))),
        (&BigQueryDialect {} as &dyn Dialect, include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/conditional-input-type-bigquery.sql"))),
        (&ClickHouseDialect {} as &dyn Dialect, include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/conditional-input-type-clickhouse.sql"))),
    ] {
        let statements = sqlparser::parser::Parser::parse_sql(dialect, sql).unwrap();
        let mut first = First(None);
        let _ = statements.visit(&mut first);
        assert!(super::input_depends_on_data_at(&Expr::Function(first.0.unwrap()), None));
    }
}
