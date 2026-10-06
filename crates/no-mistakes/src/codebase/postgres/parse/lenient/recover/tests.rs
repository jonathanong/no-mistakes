use super::{peel_do_body, recover_schema_ddl, schema_ddl_start};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

#[test]
fn recovers_only_complete_postgres_partition_transitions() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-parse-controls.sql"),
    )
    .unwrap();
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_err());
    let statements = crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql);
    assert_eq!(statements.len(), 8);
    let changes: Vec<_> = statements
        .iter()
        .filter_map(|statement| match statement {
            sqlparser::ast::Statement::AlterTable(table) => table.operations.first(),
            _ => None,
        })
        .collect();
    assert!(matches!(
        changes.as_slice(),
        [
            sqlparser::ast::AlterTableOperation::DetachPartition { .. },
            sqlparser::ast::AlterTableOperation::DetachPartition { .. },
            sqlparser::ast::AlterTableOperation::DetachPartition { .. },
            sqlparser::ast::AlterTableOperation::AttachPartition { .. },
            sqlparser::ast::AlterTableOperation::AttachPartition { .. },
            sqlparser::ast::AlterTableOperation::AttachPartition { .. },
            sqlparser::ast::AlterTableOperation::AttachPartition { .. },
            sqlparser::ast::AlterTableOperation::AddColumn { .. }
        ]
    ));
    let ordinary = sql
        .lines()
        .find(|line| line.starts_with("ALTER TABLE accounts ADD COLUMN"))
        .unwrap();
    assert!(super::partition::recover_partition_change(&tokens(ordinary), None, true).is_none());
}

#[test]
fn recovers_only_structurally_valid_partition_bounds() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-bound-controls.sql"),
    )
    .unwrap();
    let statements = crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql);
    assert_eq!(statements.len(), 6);
    assert!(statements.iter().all(|statement| matches!(
        statement,
        sqlparser::ast::Statement::AlterTable(table)
            if matches!(table.operations.as_slice(), [sqlparser::ast::AlterTableOperation::AttachPartition { .. }])
    )));
}

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
fn peel_do_body_accepts_only_complete_trailing_plpgsql_language() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-do-language-controls.sql"),
    )
    .unwrap();
    let mut forms = sql.lines().filter(|line| line.starts_with("DO "));
    let body = |line: &str| peel_do_body(&tokens(line.trim_end_matches(';')));
    assert_eq!(body(forms.next().unwrap()), Some(" SELECT 1 ".to_string()));
    assert!(forms.all(|line| body(line).is_none()));
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
    assert!(schema_ddl_start(&tokens("BEGIN CREATE VIEW v AS SELECT 1")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN CREATE MATERIALIZED VIEW v AS SELECT 1")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN TRUNCATE TABLE t")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN DROP VIEW v")).is_some());
    assert!(schema_ddl_start(&tokens("BEGIN DROP MATERIALIZED VIEW v")).is_some());
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
    let parsed = recover_schema_ddl(
        &tokens("IF THEN ALTER TABLE t ADD CONSTRAINT c CHECK (true) NOT VALID"),
        None,
        true,
    )
    .expect("alter");
    assert!(matches!(parsed, sqlparser::ast::Statement::AlterTable(_)));
    assert!(recover_schema_ddl(&tokens("IF THEN ALTER TABLE"), None, true).is_none());
    assert!(matches!(
        recover_schema_ddl(
            &tokens("IF THEN CREATE UNIQUE INDEX t_id ON t (id)"),
            None,
            true
        )
        .expect("index"),
        sqlparser::ast::Statement::CreateIndex(_)
    ));
}

#[test]
fn recover_chr_concatenations_as_sql() {
    let sql =
        "chr(85)||chr(80)||chr(68)||chr(65)||chr(84)||chr(69)||' items SET created_at = now()'";
    let expanded = super::super::expand_chr_encoded_sql(sql).expect("chr expansion");
    let statements = super::super::parse_postgres_sql_lenient(&expanded);
    assert_eq!(statements.len(), 1, "{statements:#?}");
    assert!(matches!(
        statements[0],
        sqlparser::ast::Statement::Update { .. }
    ));
    assert!(super::super::expand_chr_encoded_sql("chr (85)||' items SET x=1'").is_some());
}

#[test]
fn parse_chunks_recovers_chr_encoded_schema_after_ordinary_parse_fails() {
    let sql = "chr(67)||chr(82)||chr(69)||chr(65)||chr(84)||chr(69)||' TABLE t (id int)'";
    let statements = parse_chunks_with_sources(vec![tokens(sql)], &[], true);
    assert_eq!(statements.len(), 1, "{statements:#?}");
    assert!(matches!(
        statements[0].statement,
        sqlparser::ast::Statement::CreateTable(_)
    ));
}

#[test]
fn concatenated_strings_joins_dollar_quoted_literals() {
    assert_eq!(
        super::concatenated_strings(&tokens("$$CREATE$$ || $$ TABLE t (id int)$$")),
        Some("CREATE TABLE t (id int)".to_string())
    );
}

#[test]
fn parse_chunks_recovers_alter_when_begin_would_swallow_the_body() {
    let statements = parse_chunks_with_sources(
        vec![tokens(
        "BEGIN IF NOT EXISTS (SELECT 1) THEN ALTER TABLE t ADD CONSTRAINT c CHECK (true) NOT VALID",
        )],
        &[],
        true,
    );
    assert_eq!(statements.len(), 1, "{statements:#?}");
    assert!(matches!(
        statements[0].statement,
        sqlparser::ast::Statement::AlterTable(_)
    ));
}

#[test]
fn recovered_bodies_without_a_dollar_span_stay_put() {
    assert_eq!(
        super::locations::align_do_body(" SELECT 1", None),
        " SELECT 1"
    );
    assert_eq!(
        super::locations::align_do_body(" SELECT 1", Some(&[])),
        " SELECT 1"
    );
    assert_eq!(
        super::locations::align_chr_sql("SELECT\r\n1", None),
        "SELECT  1"
    );
}

#[test]
fn do_body_aligns_to_the_opening_dollar_quote() {
    let sql = "\nDO $$\nSELECT 1 OFFSET 2\n$$";
    let original = Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .unwrap();
    let body = peel_do_body(
        &original
            .iter()
            .map(|token| token.token.clone())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let padded = super::locations::align_do_body(&body, Some(&original));
    assert!(padded.contains("\nSELECT 1 OFFSET 2\n"), "{padded:?}");
    let select = padded.find("SELECT").unwrap();
    let prefix = &padded[..select];
    // One newline for the DO line, then padding to the column after `$$`
    // (column 4 + 2 opener bytes - 1), then the body's own leading newline.
    assert_eq!(prefix, "\n     \n");
    assert_eq!(
        super::locations::align_chr_sql("A\nB", Some(&original)),
        "\nA B"
    );
}

fn parse_chunks_with_sources(
    chunks: Vec<Vec<Token>>,
    original: &[sqlparser::tokenizer::TokenWithSpan],
    allow: bool,
) -> Vec<super::LocatedStatement> {
    super::parse_chunks_with_function_calls(chunks, original, allow)
        .into_iter()
        .filter(|located| !located.function_projection)
        .collect()
}
