use super::*;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;

#[test]
fn extracts_only_executed_static_sql_from_routine_bodies() {
    let sql = include_str!("../../../../../../../fixtures/rules/postgres-dynamic-sql/fixture.sql");
    Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .expect("fixture should tokenize");
    let extracted = extract(sql);
    assert_eq!(
        extracted
            .iter()
            .map(|item| item.sql.as_str())
            .collect::<Vec<_>>(),
        [
            "ALTER TABLE posts ADD COLUMN status text NOT NULL DEFAULT 'draft'",
            "CREATE TABLE dynamic_identifier (id uuid)",
            "CREATE TABLE semicolon_sql (value text DEFAULT 'a;b')",
            "CREATE INDEX dynamic_posts_status_idx ON posts(status)",
            "ALTER TABLE posts ADD CONSTRAINT posts_author_fk FOREIGN KEY (author_id) REFERENCES users(id)",
            "ALTER TABLE comments ADD COLUMN visible text",
            "DROP INDEX obsolete",
            "ALTER TABLE accounts ADD COLUMN generated text",
            "DROP INDEX dynamic_identifier",
        ]
    );
    assert_eq!(
        extracted.iter().map(|item| item.line).collect::<Vec<_>>(),
        [22, 26, 27, 28, 30, 39, 54, 62, 68]
    );
}

#[test]
fn recovers_direct_schema_bodies_without_duplicating_dollar_do_blocks() {
    let sql = include_str!("../../../../../../../fixtures/rules/postgres-dynamic-sql/fixture.sql");
    let bodies = schema_bodies(sql);
    assert!(bodies
        .iter()
        .any(|body| body.sql.contains("CREATE TABLE escaped_do_body")));
    assert!(bodies
        .iter()
        .any(|body| body.sql.contains("CREATE TABLE unicode_do_body")));
    assert!(bodies
        .iter()
        .any(|body| body.sql.contains("CREATE TABLE plain_do_body")));
    assert!(bodies
        .iter()
        .any(|body| body.sql.contains("CREATE TABLE concatenated_do_body")));
    assert!(bodies
        .iter()
        .any(|body| body.sql.contains("ALTER TABLE function_posts")));
    assert!(bodies
        .iter()
        .any(|body| body.sql.contains("ALTER TABLE procedure_posts")));
    assert!(!bodies
        .iter()
        .any(|body| body.sql.contains("ALTER TABLE direct_posts")));
}

#[test]
fn ignores_execute_text_in_comments_and_strings() {
    let sql = "DO $$ BEGIN\n-- EXECUTE 'CREATE TABLE comment_only (id uuid)';\nRAISE NOTICE 'EXECUTE ''CREATE TABLE string_only (id uuid)''';\nEND $$;";
    assert!(extract(sql).is_empty());
}

#[test]
fn skips_non_plpgsql_and_mismatched_dollar_bodies() {
    let sql_function = "CREATE FUNCTION sql_body() RETURNS void LANGUAGE sql AS $sql$\nSELECT 'EXECUTE ''CREATE TABLE ignored (id uuid)''';\n$sql$;";
    assert!(extract(sql_function).is_empty());

    let mismatched = "DO $open$ BEGIN EXECUTE 'CREATE TABLE mismatched (id uuid)'; END $close$;";
    assert!(extract(mismatched).is_empty());
}

#[test]
fn does_not_cross_a_single_quoted_routine_body_to_find_a_dollar_body() {
    let sql = "CREATE FUNCTION sql_body() RETURNS void LANGUAGE sql AS 'SELECT 1';\n\
               CREATE FUNCTION plpgsql_body() RETURNS void LANGUAGE plpgsql AS $body$\n\
               BEGIN\n  ddl := 'CREATE TABLE visible (id uuid)';\n  EXECUTE ddl;\nEND\n$body$;";
    let extracted = extract(sql);
    assert_eq!(extracted.len(), 1);
    assert_eq!(extracted[0].sql, "CREATE TABLE visible (id uuid)");
}

#[test]
fn ignores_runtime_concatenation_and_keeps_escaped_format_placeholders_literal() {
    let sql =
        "DO $$\nDECLARE\n  ddl text := 'ALTER TABLE posts ADD COLUMN static_value text';\nBEGIN\n\
               EXECUTE 'ALTER TABLE posts ADD COLUMN literal_value text' || runtime_part;\n\
               EXECUTE ddl || runtime_part;\n\
               EXECUTE format('CREATE TABLE %%I (id uuid)');\n\
               EXECUTE format('CREATE TABLE %1$I (id uuid)', table_name);\n\
               EXECUTE format('CREATE TABLE %-10s (id uuid)', table_name);\n\
               EXECUTE app.format('CREATE TABLE %I (id uuid)', table_name);\n\
               EXECUTE pg_catalog.format('CREATE TABLE %I (id uuid)', table_name);\nEND\n$$;";
    let extracted = extract(sql);
    assert_eq!(
        extracted
            .iter()
            .map(|item| item.sql.as_str())
            .collect::<Vec<_>>(),
        [
            "CREATE TABLE %I (id uuid)",
            "CREATE TABLE dynamic_identifier (id uuid)",
            "CREATE TABLE dynamic_value (id uuid)",
            "CREATE TABLE dynamic_identifier (id uuid)",
        ]
    );
}

#[test]
fn decodes_custom_unicode_escapes_and_maps_concatenated_lines() {
    assert_eq!(
        literal::decode_unicode_string("!D83D!DE00", '!').as_deref(),
        Some("😀")
    );
    assert!(literal::decode_unicode_string("!D83D", '!').is_none());
    assert!(literal::decode_unicode_string("!DE00", '!').is_none());
    let sql = include_str!(
        "../../../../../../../fixtures/rules/postgres-routine-string-literals/fixture.sql"
    );
    let bodies = routine::bodies(sql);
    assert_eq!(
        bodies[0].sql,
        "BEGIN \n -- 😀 C:\\temp\n CREATE TABLE custom_escape (id uuid); END"
    );
    assert_eq!(
        extract(sql)[0].sql,
        "CREATE TABLE dynamic_identifier (id uuid)"
    );
}

#[test]
fn ignores_commented_routines_and_tracks_assignments_after_control_words() {
    let sql = "/* outer /* DO $$ BEGIN EXECUTE 'CREATE TABLE ignored (id uuid)'; END $$; */ */\n\
               DO $$ BEGIN ddl := 'CREATE TABLE visible (id uuid)'; EXECUTE ddl; END $$;";
    let extracted = extract(sql);
    assert_eq!(extracted.len(), 1, "{extracted:#?}");
    assert_eq!(extracted[0].sql, "CREATE TABLE visible (id uuid)");
    assert_eq!(extracted[0].line, 2);
}

#[test]
fn retains_assignment_provenance_and_ignores_non_static_execute_expressions() {
    let sql = "DO $$\nDECLARE\n  ddl text := 'CREATE TABLE assigned (id uuid)';\nBEGIN\n  EXECUTE ddl USING value;\n  EXECUTE unknown_variable;\n  EXECUTE 'CREATE TABLE literal (id uuid)' || value;\nEND\n$$;";
    let extracted = extract(sql);
    assert_eq!(extracted.len(), 1, "{extracted:#?}");
    assert_eq!(extracted[0].sql, "CREATE TABLE assigned (id uuid)");
    assert_eq!(extracted[0].line, 3);
}

#[test]
fn rejects_invalid_literal_suffixes_and_preserves_unknown_format_directives() {
    let tokens = tokenize("'same' 'line'");
    let code = significant(&tokens);
    assert!(literal::string_expression(&code, None).is_none());

    for sql in ["'plain' UESCAPE '!'", "U&'!0041' UESCAPE identifier"] {
        let tokens = tokenize(sql);
        let code = significant(&tokens);
        assert!(literal::string_expression(&code, None).is_none());
    }
    assert_eq!(literal::normalize_format("%q %1q"), "%q %1q");
    assert!(schema_bodies("DO ' '").is_empty());
}
