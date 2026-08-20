use super::{parse_postgres_sql, parse_postgres_sql_lenient, PostgresParseError};

#[test]
fn parse_postgres_sql_accepts_create_table() {
    let statements = parse_postgres_sql("CREATE TABLE t (id int);").expect("parse");
    assert_eq!(statements.len(), 1);
}

#[test]
fn parse_postgres_sql_returns_error_for_unparseable_sql() {
    let error = parse_postgres_sql("CREATE TABLE broken (").expect_err("unparseable");
    assert!(!error.message.is_empty());
    assert!(error.to_string().contains(&error.message));
}

#[test]
fn parse_error_from_parser_error_preserves_message() {
    let error = PostgresParseError::from(sqlparser::parser::ParserError::ParserError(
        "boom".to_string(),
    ));
    assert_eq!(error.message, "boom");
    assert_eq!(error.to_string(), "boom");
}

#[test]
fn empty_sql_is_not_a_panic() {
    let statements = parse_postgres_sql("").expect("empty sql");
    assert!(statements.is_empty());
}

#[test]
fn parse_error_from_tokenizer_error_preserves_message() {
    let error = PostgresParseError::from(sqlparser::parser::ParserError::TokenizerError(
        "bad token".to_string(),
    ));
    assert_eq!(error.message, "bad token");
}

#[test]
fn parse_error_from_recursion_limit_is_not_empty() {
    let error = PostgresParseError::from(sqlparser::parser::ParserError::RecursionLimitExceeded);
    assert!(!error.message.is_empty());
}

#[test]
fn lenient_parse_skips_create_type_in_do_and_accepts_virtual_generated() {
    let statements = parse_postgres_sql_lenient(
        "DO $$ BEGIN CREATE TYPE t AS ENUM ('a'); END $$;\n\
         CREATE TABLE items (\n\
           id uuid,\n\
           created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) VIRTUAL\n\
         );",
    );
    assert_eq!(statements.len(), 1, "{statements:#?}");
    assert!(matches!(
        statements[0],
        sqlparser::ast::Statement::CreateTable(_)
    ));
}

#[test]
fn lenient_parse_recovers_alter_table_from_plpgsql_do_body() {
    let statements = parse_postgres_sql_lenient(
        "DO $$ BEGIN\n\
           IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 't_id_check') THEN\n\
             ALTER TABLE t ADD CONSTRAINT t_id_check CHECK (id IS NOT NULL) NOT VALID;\n\
           END IF;\n\
         END $$;\n\
         ALTER TABLE t VALIDATE CONSTRAINT t_id_check;",
    );
    assert_eq!(statements.len(), 2, "{statements:#?}");
    assert!(
        matches!(statements[0], sqlparser::ast::Statement::AlterTable(_)),
        "{statements:#?}"
    );
    assert!(matches!(
        statements[1],
        sqlparser::ast::Statement::AlterTable(_)
    ));
}

#[test]
fn lenient_parse_peels_language_tagged_do_and_ignores_function_bodies() {
    let language = parse_postgres_sql_lenient(
        "DO LANGUAGE plpgsql $body$\n\
           CREATE TABLE items (id int);\n\
         $body$;",
    );
    assert_eq!(language.len(), 1, "{language:#?}");
    assert!(matches!(
        language[0],
        sqlparser::ast::Statement::CreateTable(_)
    ));
    let function = parse_postgres_sql_lenient(
        "CREATE FUNCTION f() RETURNS void LANGUAGE plpgsql AS $$\n\
           BEGIN\n\
             ALTER TABLE t ADD CONSTRAINT c CHECK (true) NOT VALID;\n\
           END\n\
         $$;",
    );
    assert!(
        function
            .iter()
            .all(|statement| !matches!(statement, sqlparser::ast::Statement::AlterTable(_))),
        "{function:#?}"
    );
    let unique = parse_postgres_sql_lenient(
        "DO $$ BEGIN\n\
           IF true THEN\n\
             CREATE UNIQUE INDEX t_id ON t (id);\n\
           END IF;\n\
         END $$;",
    );
    assert!(
        unique
            .iter()
            .any(|statement| matches!(statement, sqlparser::ast::Statement::CreateIndex(_))),
        "{unique:#?}"
    );
    let dml = parse_postgres_sql_lenient("DO $$ BEGIN UPDATE items SET n = 1; END $$;");
    assert!(
        dml.iter()
            .all(|statement| !matches!(statement, sqlparser::ast::Statement::Update(_))),
        "{dml:#?}"
    );
}

#[test]
fn lenient_parse_keeps_top_level_begin_commit() {
    let statements = parse_postgres_sql_lenient("BEGIN; COMMIT;");
    assert_eq!(statements.len(), 2, "{statements:#?}");
    assert!(matches!(
        statements[0],
        sqlparser::ast::Statement::StartTransaction { .. }
    ));
    assert!(matches!(
        statements[1],
        sqlparser::ast::Statement::Commit { .. }
    ));
}

#[test]
fn lenient_parse_skips_tokenizer_failures_and_empty_chunks() {
    assert!(parse_postgres_sql_lenient("CREATE TABLE t (id text, note 'unterminated").is_empty());
    assert!(parse_postgres_sql_lenient(";;;").is_empty());
    assert!(!parse_postgres_sql_lenient("GENERATED ALWAYS; CREATE TABLE t (id int);").is_empty());
    let identity =
        parse_postgres_sql_lenient("CREATE TABLE t (id int GENERATED ALWAYS AS IDENTITY);");
    assert_eq!(identity.len(), 1, "{identity:#?}");
}
