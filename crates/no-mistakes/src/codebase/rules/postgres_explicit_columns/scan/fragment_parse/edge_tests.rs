use super::extract_fragment_stars;

#[test]
fn dml_correlated_case_window_distinct_join_on_quoted_and_token_edges() {
    let cases = [
        (
            "UPDATE narrow SET payload = (SELECT row_to_json(w.*)) FROM wide AS w",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            "DELETE FROM narrow USING wide AS w WHERE (SELECT row_to_json(w.*)) IS NOT NULL",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            "INSERT INTO wide AS w (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET payload = (SELECT row_to_json(w.*))",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            "SELECT row_to_json(CASE WHEN true THEN w.* ELSE NULL::wide END) FROM wide w",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            "SELECT row_number() OVER (PARTITION BY (SELECT count(*) FROM (SELECT * FROM a) x) ORDER BY (SELECT count(*) FROM (SELECT * FROM wide) w)) FROM narrow WHERE row_number() OVER (ORDER BY (SELECT count(*) FROM (SELECT * FROM extra) e)) > 0",
            vec![(Some("a"), None), (Some("wide"), None), (Some("extra"), None)],
        ),
        (
            "SELECT id FROM narrow WHERE (SELECT count(*) FROM (SELECT * FROM wide) w) IS DISTINCT FROM 0 AND (SELECT count(*) FROM (SELECT * FROM extra) e) IS NOT DISTINCT FROM 1",
            vec![(Some("wide"), None), (Some("extra"), None)],
        ),
        (
            "SELECT n.id FROM narrow n JOIN wide w ON (SELECT row_to_json(w.*)) IS NOT NULL",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            r#"SELECT public.wide.*, "public.wide".* FROM narrow AS "public.wide", public.wide"#,
            vec![(Some("public.wide"), None), (Some("narrow"), None)],
        ),
        (
            "SELECT to_json((SELECT * FROM wide LIMIT 1)) WHERE",
            vec![(Some("wide"), None)],
        ),
        (
            "SELECT * FROM generate_series(1, 3) AS g WHERE",
            vec![],
        ),
        (
            "SELECT g.* FROM pg_catalog.generate_series(1, 3) g(n) WHERE",
            vec![],
        ),
        (
            "SELECT (SELECT (SELECT count(wide.*)) FROM wide LIMIT 1) FROM narrow AS wide WHERE",
            vec![(Some("wide"), Some("count"))],
        ),
        (
            "WITH wide AS (SELECT 1) UPDATE wide SET id = 1 RETURNING wide.* WHERE",
            vec![(Some("wide"), None)],
        ),
        (
            "SELECT w.id FROM wide w ORDER BY (SELECT row_to_json(w.*))",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            "SELECT (SELECT row_to_json(w.*) FROM (narrow w CROSS JOIN narrow n) j LIMIT 1) FROM wide w",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            "SELECT * FROM ONLY (wide) WHERE",
            vec![(Some("wide"), None)],
        ),
        (
            "SELECT q.* FROM LATERAL (SELECT id FROM wide) q WHERE",
            vec![],
        ),
        (
            "SELECT (SELECT (SELECT q.*) FROM (SELECT 1 AS id) q) FROM wide q WHERE",
            vec![],
        ),
        (
            "SELECT * FROM narrow; SELECT id FROM wide WHERE",
            vec![(Some("narrow"), None)],
        ),
        (
            "SELECT s.doc FROM wide w CROSS JOIN LATERAL (SELECT row_to_json(w.*) AS doc) s",
            vec![(Some("wide"), Some("row_to_json"))],
        ),
        (
            r#"SELECT * FROM "audit.log" WHERE"#,
            vec![(Some(r#""audit.log""#), None)],
        ),
        (
            "SELECT g.* FROM generate_series(1, 3) WITH ORDINALITY AS g(n, ord) WHERE",
            vec![],
        ),
        (
            "SELECT v.* FROM (VALUES (1)) v WHERE",
            vec![],
        ),
    ];
    for (sql, expected) in cases {
        let stars = extract_fragment_stars(sql, None);
        let actual: Vec<_> = stars
            .iter()
            .map(|s| (s.relation.as_deref(), s.within_function.as_deref()))
            .collect();
        assert_eq!(actual, expected, "sql: {sql}");
    }

    assert!(extract_fragment_stars("SELECT * FROM wide", Some("SELECT EXISTS (")).is_empty());
    assert!(extract_fragment_stars("* 2 FROM wide", Some("SELECT quantity ")).is_empty());
}
