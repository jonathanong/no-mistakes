use super::super::{
    parse_postgres_source, PostgresSqlProceduralBlock, PostgresSqlProceduralOccurrence,
    PostgresSqlSource, PostgresSqlStatementKind,
};

fn block(sql: &str) -> PostgresSqlProceduralBlock {
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: sql.into(),
        file_name: Some("procedural-execute-into.sql".into()),
    });
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block } = &facts.statements[0].facts else {
        panic!("expected a DO block: {facts:?}");
    };
    block.clone()
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

#[test]
fn literal_execute_stops_at_an_into_target_and_keeps_the_literal_dml() {
    // INTO stays an unsupported modifier, so the block may remain incomplete.
    let parsed = block("DO $$ BEGIN EXECUTE 'UPDATE t SET v = 1 RETURNING v' INTO result; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::Other
    ));

    let parsed = block("DO $$ BEGIN EXECUTE 'INSERT INTO t VALUES (1)'; END $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::LiteralExecute { .. }
    ));

    let parsed = block("DO $$ BEGIN EXECUTE 'INSERT INTO t VALUES (1)' USING 1; END $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::LiteralExecute { .. }
    ));

    let parsed =
        block("DO $$ BEGIN EXECUTE ('UPDATE t SET v = 1 RETURNING v') INTO result; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);

    let parsed = block("DO $$ BEGIN EXECUTE format('INSERT INTO t(id) VALUES (%s)', 1); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    assert!(!parsed.complete);
}
