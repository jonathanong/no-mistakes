use super::super::{
    parse_postgres_source, PostgresSqlProceduralBlock, PostgresSqlProceduralOccurrence,
    PostgresSqlSource, PostgresSqlSpan, PostgresSqlStatementKind,
};

fn block(sql: &str) -> (String, PostgresSqlProceduralBlock) {
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: sql.into(),
        file_name: Some("procedural-occurrences.sql".into()),
    });
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block } = &facts.statements[0].facts else {
        panic!("expected a DO block: {facts:?}");
    };
    (sql.into(), block.clone())
}

fn slice<'a>(sql: &'a str, span: &PostgresSqlSpan) -> &'a str {
    &sql[span.start.offset..span.end.offset]
}

fn kinds(occurrences: &[PostgresSqlProceduralOccurrence]) -> Vec<String> {
    occurrences
        .iter()
        .map(|occurrence| {
            let nested = kinds(&occurrence.occurrences);
            if nested.is_empty() {
                format!("{:?}", occurrence.kind)
            } else {
                format!("{:?}{nested:?}", occurrence.kind)
            }
        })
        .collect()
}

fn assert_span(sql: &str, span: &PostgresSqlSpan, needle: &str) {
    let text = slice(sql, span);
    assert!(text.contains(needle), "{text:?} missing {needle}");
    assert!(span.start.offset < span.end.offset);
    assert_eq!(span.start.line, 1);
    assert!(span.start.column >= 1);
}

#[test]
fn create_type_enum_is_utility_and_raise_only_control_flow_is_not_dml() {
    let (sql, parsed) = block("DO $$ BEGIN CREATE TYPE x AS ENUM ('a'); END $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert_span(&sql, &parsed.occurrences[0].span, "CREATE TYPE x AS ENUM");
    assert!(parsed
        .statements
        .iter()
        .all(|statement| { !matches!(statement.facts, PostgresSqlStatementKind::Insert { .. }) }));

    let (sql, parsed) = block(
        "DO $$ BEGIN IF (SELECT COUNT(*) FROM t) > 0 THEN RAISE EXCEPTION 'bad'; END IF; END $$;",
    );
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.statements.is_empty());
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"ControlFlow\"]"]);
    assert_span(
        &sql,
        &parsed.occurrences[0].span,
        "IF (SELECT COUNT(*) FROM t)",
    );
    assert_span(
        &sql,
        &parsed.occurrences[0].occurrences[0].span,
        "RAISE EXCEPTION 'bad'",
    );
    assert!(!format!("{:?}", parsed.occurrences).contains("Dml"));
    assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));
}

#[test]
fn raise_between_create_tables_keeps_both_statements() {
    let (sql, parsed) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); RAISE NOTICE 'x'; CREATE TABLE b(id int); END $$;",
    );
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Utility", "ControlFlow", "Utility"]
    );
    assert_span(&sql, &parsed.occurrences[1].span, "RAISE NOTICE 'x'");
    assert!(!slice(&sql, &parsed.occurrences[1].span).contains("CREATE"));
    assert_eq!(parsed.statements.len(), 2);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(parsed.statements[1].sql.contains("CREATE TABLE b"));
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
    assert!(matches!(
        &parsed.statements[1].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
}

#[test]
fn create_beside_an_unknown_loop_stays_incomplete() {
    let (_, parsed) = block("DO $$ BEGIN CREATE TABLE a(id int); LOOP SELECT 1; END LOOP; END $$;");
    assert!(!parsed.complete);
    assert_eq!(parsed.statements.len(), 1);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(parsed.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Unsupported procedural occurrence")));
}

#[test]
fn create_beside_a_loop_keeps_the_statement_and_the_dml_diagnostic() {
    let (_, parsed) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); FOR i IN 1..2 LOOP INSERT INTO a VALUES (i); END LOOP; END $$;",
    );
    assert!(!parsed.complete);
    assert_eq!(parsed.statements.len(), 1);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(parsed
        .statements
        .iter()
        .all(|statement| !statement.sql.contains("INSERT")));
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Utility", "ControlFlow[\"Dml\"]"]
    );
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("not an executed statement")));
}

#[test]
fn loop_insert_is_visible_dml_and_dynamic_execute_stays_fail_closed() {
    let (sql, parsed) =
        block("DO $$ BEGIN FOR i IN 1..2 LOOP INSERT INTO t(id) VALUES (i); END LOOP; END $$;");
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Dml\"]"]);
    assert_span(
        &sql,
        &parsed.occurrences[0].occurrences[0].span,
        "INSERT INTO t(id) VALUES (i)",
    );
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("not an executed statement")));
    assert!(parsed.diagnostics.iter().all(|diagnostic| {
        !diagnostic
            .message
            .to_ascii_lowercase()
            .contains("executed statement")
            || diagnostic.message.contains("not an executed")
    }));

    let (_, parsed) =
        block("DO $$ BEGIN EXECUTE format('INSERT INTO t(id) VALUES (%s)', 1); END $$;");
    assert!(!parsed.complete);
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    assert!(parsed
        .statements
        .iter()
        .all(|statement| !matches!(statement.facts, PostgresSqlStatementKind::Insert { .. })));

    let (_, parsed) = block("DO $$ BEGIN LOOP EXECUTE format('SELECT 1'); END LOOP; END $$;");
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"DynamicExecute\"]"]
    );
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Dynamic EXECUTE is unknown")));
}

#[test]
fn nested_control_flow_reports_static_dml_without_reading_strings_or_comments() {
    let sql = "DO $$ BEGIN IF (SELECT COUNT(*) FROM t) > 0 THEN FOR i IN 1..2 LOOP UPDATE t SET id = i; END LOOP; ELSE DELETE FROM t; END IF; END $$;";
    let (_, parsed) = block(sql);
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"ControlFlow[\\\"Dml\\\"]\", \"Dml\"]"]
    );
    let (_, parsed) = block(
        "DO $$ BEGIN WHILE true LOOP MERGE INTO t USING s ON true WHEN MATCHED THEN DO NOTHING; END LOOP; END $$;",
    );
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Dml\"]"]);
    assert!(!parsed.complete);

    let (_, parsed) = block(
        "DO $$ BEGIN CASE WHEN true THEN INSERT INTO t(id) VALUES (1); ELSE UPDATE t SET id = 2; END CASE; END $$;",
    );
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Dml\", \"Dml\"]"]
    );

    let (_, parsed) = block(
        "DO $$ BEGIN -- INSERT INTO t VALUES (1);\n /* EXECUTE format('hidden') */ RAISE NOTICE 'EXECUTE INSERT'; END $$;",
    );
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow"]);
    assert!(parsed.diagnostics.is_empty());
}

#[test]
fn literal_execute_is_reclassified_and_dynamic_concatenation_is_not_guessed() {
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'INSERT INTO t(id) VALUES (1)'; END $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::LiteralExecute { .. }
    ));

    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'CREATE TYPE x AS ENUM (''a'')'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert!(parsed
        .diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message.contains("Dynamic EXECUTE")));

    let (_, parsed) = block("DO $$ BEGIN EXECUTE ('INSERT INTO t(id) VALUES (1)'); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE E'INSERT INTO t(id) VALUES (1)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE $cmd$INSERT INTO t(id) VALUES (1)$cmd$; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE N'INSERT INTO t(id) VALUES (1)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE U&'INSERT INTO t(id) VALUES (1)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN <<mark>> CREATE TYPE x AS ENUM ('a'); END mark; END $$;");
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("Utility")));
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'INSERT INTO t(id) ' || 'VALUES (1)'; END $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::LiteralExecute { .. }
    ));

    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'INSERT INTO t(id) ' || suffix; END $$;");
    assert!(!parsed.complete);
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    assert!(parsed
        .statements
        .iter()
        .all(|statement| { !matches!(statement.facts, PostgresSqlStatementKind::Insert { .. }) }));
    let (_, parsed) = block("DO $$ BEGIN EXECUTE command; END $$;");
    assert!(!parsed.complete);
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
}

#[test]
fn utility_blocks_stay_distinct_from_hidden_dml_and_malformed_procedures() {
    let (_, parsed) = block("DO $$ BEGIN CREATE TABLE kept (id integer); END $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));

    let (_, parsed) = block("DO $$ BEGIN CREATE TABLE t (\"insert\" integer); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert!(parsed.complete, "{:?}", parsed.diagnostics);

    let (sql, parsed) = block("DO 'BEGIN CREATE TYPE x AS ENUM (''a''); END';");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert_span(
        &sql,
        &parsed.occurrences[0].span,
        "CREATE TYPE x AS ENUM (''a'')",
    );

    let (_, parsed) = block(
        "DO $$ BEGIN DECLARE x integer; ALTER TABLE hidden_table ADD COLUMN hidden integer; END $$;",
    );
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(parsed.diagnostics[0].message.contains("unsupported"));
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("Dml") || kind.contains("Utility")));

    let (_, parsed) = block("DO $$ SELECT 1; $$;");
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(parsed.diagnostics.len(), 1);

    let (_, parsed) = block("DO $$ BEGIN CREATE TYPE x AS ENUM ('a'); $$;");
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(parsed.diagnostics.len(), 1);
    assert!(parsed.diagnostics[0].span.is_some());
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("Utility")));
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("Unknown")));

    let (_, parsed) = block("DO LANGUAGE sql $$ BEGIN CREATE TABLE t (id integer); END $$;");
    assert!(!parsed.complete);
    assert!(parsed.occurrences.is_empty());
    assert!(parsed.statements.is_empty());
}

#[test]
fn exception_handlers_nested_do_and_cte_dml_keep_occurrence_kinds() {
    let (_, parsed) = block(
        "DO $$ BEGIN INSERT INTO t(id) VALUES (1); EXCEPTION WHEN unique_violation THEN DELETE FROM t; END $$;",
    );
    assert!(!parsed.complete);
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("not an executed statement")));
    assert_eq!(parsed.statements.len(), 1);
    assert!(parsed.statements[0]
        .sql
        .contains("INSERT INTO t(id) VALUES (1)"));
    assert!(parsed
        .statements
        .iter()
        .all(|statement| !statement.sql.contains("DELETE")));
    assert_eq!(kinds(&parsed.occurrences), ["Dml", "ControlFlow[\"Dml\"]"]);

    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: "DO $$ BEGIN DO $inner$ BEGIN CREATE TYPE y AS ENUM ('b'); END $inner$; END $$;"
            .into(),
        file_name: None,
    });
    let PostgresSqlStatementKind::DoBlock { block: outer } = &facts.statements[0].facts else {
        panic!("{facts:?}");
    };
    assert_eq!(kinds(&outer.occurrences), ["ControlFlow"]);
    let PostgresSqlStatementKind::DoBlock { block: inner } = &outer.statements[0].facts else {
        panic!("nested DO: {outer:?}");
    };
    assert_eq!(kinds(&inner.occurrences), ["Utility"]);
    assert!(inner.complete, "{:?}", inner.diagnostics);

    let (_, parsed) = block(
        "DO $$ BEGIN WITH c AS (INSERT INTO t(id) VALUES (1) RETURNING id) SELECT * FROM c; END $$;",
    );
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN WITH c AS (SELECT 1) SELECT * FROM c; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    assert!(parsed.complete, "{:?}", parsed.diagnostics);

    let (_, parsed) = block("DO $$ BEGIN IF true THEN EXECUTE command; END IF; END $$;");
    assert!(!parsed.complete);
    assert!(kinds(&parsed.occurrences)[0].contains("DynamicExecute"));

    let mut sql = String::from("DO $$ BEGIN ");
    for _ in 0..70 {
        sql.push_str("LOOP ");
    }
    sql.push_str("INSERT INTO t(id) VALUES (1); ");
    for _ in 0..70 {
        sql.push_str("END LOOP; ");
    }
    sql.push_str("END $$;");
    let (_, parsed) = block(&sql);
    assert!(!parsed.complete);
    assert!(format!("{:?}", parsed.occurrences).contains("Dml"));
    assert!(format!("{:?}", parsed.occurrences).contains("Unknown"));
}

#[test]
fn partial_forms_keep_distinct_occurrence_kinds() {
    let (_, parsed) = block("DO $$ BEGIN CREATE TABLE t(id integer); ; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert!(parsed.complete, "{:?}", parsed.diagnostics);

    let (_, parsed) = block("DO $$ BEGIN LOOP SELECT 1; END LOOP; END $$;");
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(parsed.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Unsupported procedural occurrence")));

    let (_, parsed) = block("DO $$ BEGIN IF true RAISE NOTICE 'a'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown", "Unknown"]);
    let (_, parsed) = block("DO $$ BEGIN IF true THEN RAISE NOTICE 'a'; END $$;");
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Unknown[\"ControlFlow\"]", "Unknown"]
    );
    let (_, parsed) = block("DO $$ BEGIN CASE WHEN true RAISE NOTICE 'a'; END CASE; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown", "Unknown"]);
    let (_, parsed) = block("DO $$ BEGIN CASE WHEN true THEN RAISE NOTICE 'a'; END $$;");
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Unknown[\"ControlFlow\"]", "Unknown"]
    );
    let (_, parsed) = block("DO $$ BEGIN FOR i IN 1..2 INSERT INTO t(id) VALUES (i); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown[\"Dml\"]", "Unknown"]);

    let (_, parsed) = block(
        "DO $$ BEGIN BEGIN INSERT INTO t(id) VALUES (1); EXCEPTION WHEN unique_violation THEN DELETE FROM t; END; END $$;",
    );
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Dml\", \"ControlFlow[\\\"Dml\\\"]\"]"]
    );
    assert!(parsed.statements.is_empty());
    let (_, parsed) = block("DO $$ BEGIN BEGIN INSERT INTO t(id) VALUES (1); END IF; END $$;");
    assert!(kinds(&parsed.occurrences)[0].contains("Unknown"));
    let (_, parsed) = block(
        "DO $$ BEGIN INSERT INTO t(id) VALUES (1); EXCEPTION WHEN unique_violation DELETE FROM t; END $$;",
    );
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("Unknown")));

    let (_, parsed) = block("DO $$ BEGIN IF INSERT THEN RAISE NOTICE 'a'; END IF; END $$;");
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Dml\", \"ControlFlow\"]"]
    );
    let (_, parsed) = block("DO $$ BEGIN IF EXECUTE THEN RAISE NOTICE 'a'; END IF; END $$;");
    assert!(kinds(&parsed.occurrences)[0].contains("DynamicExecute"));
    assert!(!parsed.complete);
    let (_, parsed) = block(
        "DO $$ BEGIN IF CASE WHEN true THEN 1 ELSE 0 END > 0 THEN RAISE NOTICE 'ok'; END IF; END $$;",
    );
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"ControlFlow\"]"]);
    assert!(parsed.complete, "{:?}", parsed.diagnostics);

    let (_, parsed) =
        block("DO $$ BEGIN WITH c AS (SELECT 1) INSERT INTO t(id) VALUES (1); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN WITH c AS (SELECT 1) EXECUTE command; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    assert!(!parsed.complete);
    let (_, parsed) = block("DO $$ BEGIN RAISE NOTICE INSERT; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Dml\"]"]);
    let (_, parsed) = block("DO $$ BEGIN RAISE NOTICE EXECUTE; END $$;");
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"DynamicExecute\"]"]
    );
    let (_, parsed) = block("DO $$ BEGIN CREATE TYPE x AS ENUM ('a'); EXECUTE $$;");
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("DynamicExecute")));
    let (_, parsed) =
        block("DO $$ BEGIN CREATE TABLE t(id integer); END IF; EXECUTE command; END $$;");
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("Unknown") && kind.contains("DynamicExecute")));

    let (_, parsed) = block("DO $$ BEGIN EXECUTE ''; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'SELECT 1'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    assert!(parsed
        .statements
        .iter()
        .all(|statement| !matches!(statement.facts, PostgresSqlStatementKind::Insert { .. })));
    let (_, parsed) =
        block("DO $$ BEGIN EXECUTE 'IF true THEN CREATE TABLE kept(id integer); END IF;'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'EXECUTE command'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE ('INSERT INTO t(id) VALUES (1)' extra); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE $q$U&'bad\\zzzz'$q$; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);

    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: "DO $$ BEGIN U&'bad\\zzzz'; END $$;".into(),
        file_name: Some("procedural-occurrences.sql".into()),
    });
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block: parsed } = &facts.statements[0].facts else {
        panic!("{facts:?}");
    };
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);

    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: "DO $$ <<mark>> BEGIN CREATE TYPE x AS ENUM ('a'); END $$;".into(),
        file_name: Some("procedural-occurrences.sql".into()),
    });
    let PostgresSqlStatementKind::DoBlock { block: parsed } = &facts.statements[0].facts else {
        panic!("{facts:?}");
    };
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unsupported")));
}
