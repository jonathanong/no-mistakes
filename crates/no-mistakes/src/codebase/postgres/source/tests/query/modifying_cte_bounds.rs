use super::*;

fn parse(sql: &str) -> PostgresSqlFacts {
    parse_postgres_source(&PostgresSqlSource {
        sql: sql.to_owned(),
        file_name: None,
    })
}

fn assert_reparse(sql: &str) {
    let again = parse(sql);
    assert!(
        again.diagnostics.is_empty(),
        "{:?}\n{sql}",
        again.diagnostics
    );
    assert_eq!(again.statements.len(), 1, "{sql}");
    assert_eq!(again.statements[0].sql, sql);
}

fn nested(sql: &str) -> Vec<PostgresSqlQueryStatement> {
    let facts = parse(sql);
    assert!(
        facts.diagnostics.is_empty(),
        "{:?}\n{sql}",
        facts.diagnostics
    );
    let PostgresSqlStatementKind::Select { query } = &facts.statements[0].facts else {
        panic!("select: {sql}");
    };
    assert!(query.complete, "{:?}\n{sql}", query.unsupported);
    let mut previous = 0usize;
    for child in &query.nested_statements {
        assert!(child.complete, "{:?} {}", child.unsupported, child.sql);
        let span = child.span.as_ref().expect("span");
        assert_eq!(&sql[span.start.offset..span.end.offset], child.sql);
        assert!(span.start.offset >= previous, "{}", child.sql);
        assert!(span.end.offset > span.start.offset);
        previous = span.end.offset;
        assert_reparse(&child.sql);
    }
    query.nested_statements.clone()
}

#[test]
fn partial_parser_spans_expand_to_the_exact_nested_statement() {
    let values =
        nested("WITH a AS (INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING) SELECT 1;");
    assert_eq!(
        values[0].sql,
        "INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING"
    );
    let exists =
        nested("WITH a AS (INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1)) SELECT 1;");
    assert_eq!(
        exists[0].sql,
        "INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1)"
    );
    let nested_exists =
        nested("WITH a AS (INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT (1))) SELECT 1;");
    assert_eq!(
        nested_exists[0].sql,
        "INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT (1))"
    );
    let conflict =
        nested("WITH a AS (INSERT INTO t(id, v) SELECT 1, now() ON CONFLICT DO NOTHING) SELECT 1;");
    assert_eq!(
        conflict[0].sql,
        "INSERT INTO t(id, v) SELECT 1, now() ON CONFLICT DO NOTHING"
    );
    let update = nested("WITH a AS (UPDATE t SET id = 1 WHERE NOT EXISTS (SELECT 1)) SELECT 1;");
    assert_eq!(
        update[0].sql,
        "UPDATE t SET id = 1 WHERE NOT EXISTS (SELECT 1)"
    );
    let delete = nested("WITH a AS (DELETE FROM t WHERE NOT EXISTS (SELECT 1)) SELECT 1;");
    assert_eq!(delete[0].sql, "DELETE FROM t WHERE NOT EXISTS (SELECT 1)");
}

#[test]
fn quoted_identifiers_comments_and_returning_keep_exact_slices() {
    let quoted = nested(
        "WITH a AS (INSERT INTO \"t)\"(id) VALUES (1) ON CONFLICT (\"i)d\") /* arbiter */ DO NOTHING) SELECT 1;",
    );
    assert_eq!(
        quoted[0].sql,
        "INSERT INTO \"t)\"(id) VALUES (1) ON CONFLICT (\"i)d\") /* arbiter */ DO NOTHING"
    );
    let returning = nested(
        "WITH a AS (INSERT INTO t(id) VALUES (1) /* kept */ ON CONFLICT (id) DO NOTHING RETURNING id /* tail */) SELECT 1;",
    );
    assert_eq!(
        returning[0].sql,
        "INSERT INTO t(id) VALUES (1) /* kept */ ON CONFLICT (id) DO NOTHING RETURNING id"
    );
    assert!(!returning[0].sql.contains("tail"));
    let crlf =
        "WITH a AS (INSERT INTO t(id)\r\nVALUES (1) ON CONFLICT (id) DO NOTHING)\r\nSELECT 1;";
    let children = nested(crlf);
    assert_eq!(
        children[0].sql,
        "INSERT INTO t(id)\r\nVALUES (1) ON CONFLICT (id) DO NOTHING"
    );
    let defaults = nested("WITH a AS (INSERT INTO target DEFAULT VALUES) SELECT 1;");
    assert_eq!(defaults[0].sql, "INSERT INTO target DEFAULT VALUES");
}

#[test]
fn sibling_cte_statements_are_ordered_exact_and_non_overlapping() {
    let children = nested(
        "WITH a(x) AS NOT MATERIALIZED (INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING),\n\
         b AS (INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1 /* in */))\n\
         SELECT 1;",
    );
    assert_eq!(
        children[0].sql,
        "INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING"
    );
    assert_eq!(
        children[1].sql,
        "INSERT INTO t(id) SELECT 1 WHERE NOT EXISTS (SELECT 1 /* in */)"
    );
    let sql = "WITH outer AS (\n  WITH inner AS (SELECT 1 AS id)\n  INSERT INTO t(id) SELECT id FROM inner ON CONFLICT (id) DO NOTHING\n) SELECT 1;";
    let children = nested(sql);
    assert_eq!(children.len(), 1);
    assert_eq!(
        children[0].sql,
        "INSERT INTO t(id) SELECT id FROM inner ON CONFLICT (id) DO NOTHING"
    );
    assert!(!children[0].sql.contains("WITH"));
}

#[test]
fn do_block_nested_statements_use_the_same_exact_slice() {
    let sql = "DO $$ BEGIN WITH a AS (INSERT INTO \"Schéma\"(id) VALUES (1) ON CONFLICT (id) /* café */ DO NOTHING) SELECT 1; END $$;";
    let facts = parse(sql);
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block } = &facts.statements[0].facts else {
        panic!("do");
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let PostgresSqlStatementKind::Select { query } = &block.statements[0].facts else {
        panic!("select");
    };
    assert!(query.complete, "{:?}", query.unsupported);
    let child = &query.nested_statements[0];
    let span = child.span.as_ref().unwrap();
    assert_eq!(&sql[span.start.offset..span.end.offset], child.sql);
    assert_eq!(
        child.sql,
        "INSERT INTO \"Schéma\"(id) VALUES (1) ON CONFLICT (id) /* café */ DO NOTHING"
    );
    assert!(child.complete, "{:?}", child.unsupported);
    assert_reparse(&child.sql);
}

#[test]
fn existing_returning_slices_stay_complete_and_reparse() {
    let sql = fixture("query-modifying-ctes.sql");
    let facts = facts("query-modifying-ctes.sql");
    let first = match &facts.statements[0].facts {
        PostgresSqlStatementKind::Select { query } => &query.nested_statements[0],
        _ => panic!("select"),
    };
    assert_eq!(
        first.sql,
        "INSERT INTO \"Schéma\".\"Target\" (id) VALUES (1), (2) /* kept */ RETURNING id AS \"ID\""
    );
    let defaults = match &facts.statements[1].facts {
        PostgresSqlStatementKind::Select { query } => &query.nested_statements[0],
        _ => panic!("select"),
    };
    assert_eq!(defaults.sql, "INSERT INTO target DEFAULT VALUES");
    let conflict = match &facts.statements[3].facts {
        PostgresSqlStatementKind::Select { query } => &query.nested_statements[1],
        _ => panic!("select"),
    };
    assert_eq!(
        conflict.sql,
        "INSERT INTO target (id) VALUES (5) ON CONFLICT (id) DO NOTHING RETURNING target.*"
    );
    for statement in &facts.statements {
        let PostgresSqlStatementKind::Select { query } = &statement.facts else {
            continue;
        };
        for child in &query.nested_statements {
            if !child.complete {
                continue;
            }
            let span = child.span.as_ref().unwrap();
            assert_eq!(&sql[span.start.offset..span.end.offset], child.sql);
            assert_reparse(&child.sql);
        }
    }
}
