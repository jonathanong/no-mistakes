use super::super::simple_schema_rule::{compile_options, Options};

// Saved-fixture output cases moved to cli_postgres_snapshot_constraint_validate.
// Keep the private option invariant here; CLI output cannot expose it.
#[test]
fn option_defaults_use_schema_sql_include() {
    let compiled = compile_options(&Options::default());
    assert_eq!(
        compiled.schema.sql_include,
        crate::codebase::postgres::PostgresSchemaOptions::default().sql_include
    );
}
