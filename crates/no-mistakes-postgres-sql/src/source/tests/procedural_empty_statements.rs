use super::super::{
    parse_postgres_source, PostgresSqlProceduralBlock, PostgresSqlProceduralOccurrence,
    PostgresSqlSource, PostgresSqlSpan, PostgresSqlStatementKind,
};

fn block(sql: &str) -> (String, PostgresSqlProceduralBlock) {
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: sql.into(),
        file_name: Some("procedural-empty-statements.sql".into()),
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

#[test]
fn consecutive_empty_statements_keep_the_following_dml_visible() {
    // A second semicolon must stay empty. It must not become an `unknown`
    // statement whose scan consumes the INSERT.
    let needle = "INSERT INTO t VALUES (1)";
    for sql in [
        "DO $$ BEGIN ;; INSERT INTO t VALUES (1); END $$;",
        "DO $$ BEGIN INSERT INTO t VALUES (1); END $$;",
        "DO $$ BEGIN ; ; INSERT INTO t VALUES (1); END $$;",
    ] {
        let (sql, parsed) = block(sql);
        assert_eq!(kinds(&parsed.occurrences), ["Dml"], "{sql}");
        let text = slice(&sql, &parsed.occurrences[0].span);
        assert!(text.contains(needle), "{text:?}");
        assert!(text.starts_with("INSERT"), "{text:?}");
        assert!(parsed.occurrences[0].occurrences.is_empty());
        assert!(!format!("{:?}", parsed.occurrences).contains("Unknown"));
    }
}
