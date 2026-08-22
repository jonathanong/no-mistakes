use super::{peel_do_body, recover_schema_ddl, schema_ddl_start};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

fn tokens(sql: &str) -> Vec<Token> {
    Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize()
        .expect("tokenize")
}

#[test]
fn peel_do_body_reads_dollar_quote_and_optional_language() {
    assert_eq!(
        peel_do_body(&tokens("DO $$ ALTER TABLE t ADD COLUMN id int; $$")),
        Some(" ALTER TABLE t ADD COLUMN id int; ".to_string())
    );
    assert_eq!(
        peel_do_body(&tokens("DO LANGUAGE plpgsql $body$ SELECT 1; $body$")),
        Some(" SELECT 1; ".to_string())
    );
}

#[test]
fn peel_do_body_rejects_non_do_and_malformed_language() {
    assert!(peel_do_body(&tokens("CREATE TABLE t (id int)")).is_none());
    assert!(peel_do_body(&tokens("DO")).is_none());
    assert!(peel_do_body(&tokens("DO LANGUAGE")).is_none());
    assert!(peel_do_body(&tokens("DO LANGUAGE ;")).is_none());
    assert!(peel_do_body(&tokens("DO LANGUAGE plpgsql")).is_none());
    assert!(peel_do_body(&tokens("DO LANGUAGE plpgsql;")).is_none());
    assert!(peel_do_body(&tokens("DO plpgsql")).is_none());
    assert!(peel_do_body(&tokens(
        "DO $$ ALTER TABLE t ADD COLUMN id int $$ unexpected"
    ))
    .is_none());
    assert!(peel_do_body(&[]).is_none());
}

#[test]
fn schema_ddl_start_finds_alter_create_and_unique_index() {
    assert!(schema_ddl_start(&tokens(
        "IF THEN ALTER TABLE t ADD CONSTRAINT c CHECK (true) NOT VALID"
    ))
    .is_some());
    assert!(schema_ddl_start(&tokens("BEGIN CREATE TABLE t (id int)")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN CREATE INDEX t_id ON t (id)")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN CREATE UNIQUE INDEX t_id ON t (id)")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN DROP INDEX idx_t")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN DROP TABLE t")).is_some());
}

#[test]
fn schema_ddl_start_skips_non_schema_ddl() {
    assert!(schema_ddl_start(&tokens("ALTER INDEX t_id RENAME TO t_id2")).is_none());
    assert!(schema_ddl_start(&tokens("CREATE TYPE t AS ENUM ('a')")).is_none());
    assert!(schema_ddl_start(&tokens("CREATE UNIQUE")).is_none());
    assert!(schema_ddl_start(&tokens("CREATE")).is_none());
    assert!(schema_ddl_start(&tokens("ALTER")).is_none());
    assert!(schema_ddl_start(&tokens("SELECT 1")).is_none());
    assert!(schema_ddl_start(&tokens("DROP TYPE t")).is_none());
    assert!(schema_ddl_start(&tokens("DROP")).is_none());
}

#[test]
fn recover_schema_ddl_parses_or_skips_trailing_junk() {
    let parsed = recover_schema_ddl(&tokens(
        "IF THEN ALTER TABLE t ADD CONSTRAINT c CHECK (true) NOT VALID",
    ))
    .expect("alter");
    assert!(matches!(parsed, sqlparser::ast::Statement::AlterTable(_)));
    assert!(recover_schema_ddl(&tokens("IF THEN ALTER TABLE")).is_none());
    assert!(matches!(
        recover_schema_ddl(&tokens("IF THEN CREATE UNIQUE INDEX t_id ON t (id)")).expect("index"),
        sqlparser::ast::Statement::CreateIndex(_)
    ));
}

#[test]
fn recover_chr_concatenations_as_sql() {
    let sql =
        "chr(85)||chr(80)||chr(68)||chr(65)||chr(84)||chr(69)||' items SET created_at = now()'";
    let statements = super::super::parse_postgres_sql_lenient(sql);
    assert_eq!(statements.len(), 1, "{statements:#?}");
    assert!(matches!(
        statements[0],
        sqlparser::ast::Statement::Update { .. }
    ));
}

#[test]
fn parse_chunks_recovers_alter_when_begin_would_swallow_the_body() {
    let statements = super::parse_chunks(vec![tokens(
        "BEGIN IF NOT EXISTS (SELECT 1) THEN ALTER TABLE t ADD CONSTRAINT c CHECK (true) NOT VALID",
    )]);
    assert_eq!(statements.len(), 1, "{statements:#?}");
    assert!(matches!(
        statements[0],
        sqlparser::ast::Statement::AlterTable(_)
    ));
}
