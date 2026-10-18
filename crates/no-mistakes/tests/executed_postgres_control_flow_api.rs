use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, parse_postgres_sql, EmbeddedSqlOptions,
    ExecutedPostgresSource, ExecutedPostgresSql, TrustedSqlTag,
};
use std::path::PathBuf;

fn extract(name: &str) -> Vec<ExecutedPostgresSql> {
    extract_with_options(
        name,
        &EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
            TrustedSqlTag {
                module: "@example/db".into(),
                name: "sql".into(),
            },
        ]),
    )
}

fn extract_with_options(name: &str, options: &EmbeddedSqlOptions) -> Vec<ExecutedPostgresSql> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source")
        .join(name);
    extract_executed_postgres_sql_with_imports(
        &[ExecutedPostgresSource {
            file_name: name.into(),
            source: std::fs::read_to_string(path).unwrap(),
        }],
        options,
        &[],
    )
    .unwrap()
}

// Consumers must inspect every parsed alternative, including nested statements;
// SELECT/WITH prefix tests would wrongly accept the modifying CTE in this fixture.
fn read_statement_shape(rows: &[&ExecutedPostgresSql]) -> bool {
    use sqlparser::ast::{Query, Select, Statement, Visit, Visitor};
    use std::ops::ControlFlow;
    struct Reads;
    impl Visitor for Reads {
        type Break = ();
        fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
            if matches!(statement, Statement::Query(_)) {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            if query.locks.is_empty() {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
        fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<()> {
            if select.into.is_none() {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
    }
    !rows.is_empty()
        && rows.iter().all(|row| {
            if !row.complete && !row.only_value_placeholders {
                return false;
            }
            row.sql_text
                .as_ref()
                .and_then(|text| parse_postgres_sql(text).ok())
                .is_some_and(|statements| {
                    !statements.is_empty()
                        && matches!(statements.visit(&mut Reads), ControlFlow::Continue(()))
                })
        })
}

#[path = "executed_postgres_control_flow_api/part1.rs"]
mod part1;
#[path = "executed_postgres_control_flow_api/part2.rs"]
mod part2;
#[path = "executed_postgres_control_flow_api/part3.rs"]
mod part3;
