use super::{extract_fragment_stars, parse_context};

#[test]
fn dml_union_table_reader_clause_and_token_scope_edges() {
    let cases = [
        (
            "SELECT id FROM narrow UNION TABLE wide; SELECT id FROM narrow UNION TABLE public.wide; WITH c AS (SELECT id FROM wide) SELECT id FROM narrow UNION TABLE c",
            vec![(Some("wide"), None), (Some("public.wide"), None)],
        ),
        (
            "INSERT INTO wide DEFAULT VALUES RETURNING *",
            vec![(Some("wide"), None)],
        ),
        (
            "SELECT CASE WHEN true THEN w.* END FROM wide w",
            vec![(Some("wide"), None)],
        ),
        ("SELECT * FROM wide;", vec![(Some("wide"), None)]),
        ("SELECT * FROM public.1 WHERE", vec![(Some("public"), None)]),
        (
            "SELECT * FROM ONLY (wide WHERE",
            vec![(Some("wide"), None)],
        ),
        ("SELECT * FROM wide AS 1 WHERE", vec![(Some("wide"), None)]),
        ("SELECT .* FROM wide WHERE", vec![(None, None)]),
        ("SELECT 1.a.* FROM wide WHERE", vec![(None, None)]),
        ("SELECT * FROM 1 WHERE", vec![(None, None)]),
        ("SELECT * FROM pg_catalog.1(1) WHERE", vec![(Some("pg_catalog"), None)]),
        ("SELECT * FROM fn(1) WITH x WHERE", vec![]),
        ("SELECT * FROM fn(1) AS f WHERE", vec![]),
        ("SELECT * FROM fn(1) WHERE", vec![]),
        ("SELECT ) * FROM wide WHERE", vec![]),
        ("SELECT (1) * FROM wide WHERE", vec![]),
        ("SELECT (*) FROM wide WHERE", vec![(Some("wide"), None)]),
        (
            "SELECT 1 + (*) FROM wide WHERE",
            vec![(Some("wide"), None)],
        ),
        (
            "UPDATE wide SET a = 1 RETURNING other.* WHERE",
            vec![(None, None)],
        ),
        (
            "INSERT INTO wide (id) SELECT other.* WHERE",
            vec![(None, None)],
        ),
        (
            "SELECT (SELECT other.* FROM narrow) FROM (SELECT 1) d WHERE",
            vec![(None, None)],
        ),
        ("SELECT * FROM ONLY (", vec![(None, None)]),
    ];
    for (sql, expected) in cases {
        let stars = extract_fragment_stars(sql, None);
        let actual: Vec<_> = stars
            .iter()
            .map(|s| (s.relation.as_deref(), s.within_function.as_deref()))
            .collect();
        assert_eq!(actual, expected, "sql: {sql}");
    }

    assert!(parse_context("WITH ").ctes.is_empty());
    assert!(parse_context("WITH c ").ctes.is_empty());
    assert!(extract_fragment_stars(") * FROM wide WHERE", Some("SELECT ")).is_empty());
    assert!(extract_fragment_stars("(1) * FROM wide WHERE", Some("SELECT ")).is_empty());
    assert_eq!(
        extract_fragment_stars("(*) FROM wide WHERE", Some("SELECT ")).len(),
        1
    );
    assert!(extract_fragment_stars("* FROM wide", Some("SELECT ) ")).is_empty());
}
