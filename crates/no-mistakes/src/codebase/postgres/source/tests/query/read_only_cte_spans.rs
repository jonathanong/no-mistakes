use super::*;

#[test]
fn read_only_cte_scopes_cover_their_complete_source() {
    let source = fixture("query-readonly-cte-spans.sql");
    let queries = queries("query-readonly-cte-spans.sql");
    assert_eq!(queries.len(), 6);
    for query in &queries {
        assert!(query.complete, "{:?}", query.unsupported);
        assert!(query.unsupported.is_empty());
    }
    let slices = |query: &PostgresSqlQuery| {
        query
            .scopes
            .iter()
            .map(|scope| {
                let span = scope.span.as_ref().expect("complete scope has a span");
                source[span.start.offset..span.end.offset].to_owned()
            })
            .collect::<Vec<_>>()
    };
    let simple = slices(&queries[0]);
    assert!(simple.contains(&"SELECT now()".to_owned()), "{simple:?}");
    assert!(
        simple.contains(&"WITH s AS (SELECT now()) SELECT count(*) FROM s".to_owned()),
        "{simple:?}"
    );

    let nested = slices(&queries[1]);
    for expected in [
        "SELECT 'é' AS word",
        "SELECT lower('é')",
        "WITH nested AS (SELECT lower('é')) SELECT * FROM nested",
        "WITH first AS (SELECT 'é' AS word), second AS (WITH nested AS (SELECT lower('é')) SELECT * FROM nested) SELECT count(*) FROM first, second",
    ] {
        assert!(nested.contains(&expected.to_owned()), "missing {expected:?}: {nested:?}");
    }

    assert!(slices(&queries[2]).contains(&"SELECT now()".to_owned()));
    let projected = slices(&queries[3]);
    assert!(
        projected.contains(&"SELECT now()".to_owned()),
        "{projected:?}"
    );
    assert!(
        projected.contains(&"SELECT (SELECT now()) AS value".to_owned()),
        "{projected:?}"
    );

    let set = slices(&queries[4]);
    assert!(
        set.contains(&"SELECT now() UNION ALL SELECT now()".to_owned()),
        "{set:?}"
    );
    assert_eq!(
        set.iter().filter(|slice| *slice == "SELECT now()").count(),
        2,
        "{set:?}"
    );

    let ordered = slices(&queries[5]);
    assert!(
        ordered.contains(&"SELECT now() UNION ALL SELECT now() ORDER BY 1".to_owned()),
        "{ordered:?}"
    );
    assert_eq!(
        ordered
            .iter()
            .filter(|slice| *slice == "SELECT now()")
            .count(),
        2,
        "{ordered:?}"
    );
}
