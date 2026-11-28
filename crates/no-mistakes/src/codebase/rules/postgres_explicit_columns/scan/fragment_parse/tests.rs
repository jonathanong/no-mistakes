use super::{extract_fragment_stars, parse_context};

#[test]
fn ast_covers_dml_set_ops_table_and_nested_subqueries() {
    let cases = [
        ("INSERT INTO entries (id) VALUES (1) RETURNING *", vec![(true, Some("entries"))]),
        (
            "INSERT INTO entries SELECT * FROM records",
            vec![(false, Some("records"))],
        ),
        (
            "UPDATE entries e SET a = 1 FROM records r JOIN other o ON o.id = (SELECT x.* FROM extra x LIMIT 1) RETURNING e.*, r.*",
            vec![
                (false, Some("extra")),
                (true, Some("entries")),
                (true, Some("records")),
            ],
        ),
        (
            "DELETE FROM entries USING records WHERE entries.id = records.id RETURNING *",
            vec![(true, Some("entries"))],
        ),
        ("EXPLAIN SELECT * FROM entries", vec![(false, Some("entries"))]),
        ("TABLE public.entries", vec![(false, Some("public.entries"))]),
        (
            "SELECT id FROM narrow UNION ALL (SELECT * FROM entries)",
            vec![(false, Some("entries"))],
        ),
        (
            "SELECT CASE WHEN id IN (SELECT * FROM entries) AND (id BETWEEN 1 AND 2) THEN (SELECT row_to_json(r.*) FROM records r LIMIT 1) ELSE NULL END FROM narrow WHERE id IN (1, 2) AND id LIKE 'a%' AND EXISTS (SELECT * FROM ignored) HAVING count(*) > (SELECT count(w.*) FROM wide w)",
            vec![
                (false, Some("entries")),
                (false, Some("records")),
                (false, Some("wide")),
            ],
        ),
        (
            "SELECT ARRAY[1, (SELECT * FROM entries)], (1, (SELECT * FROM records)), array(SELECT * FROM third) FROM narrow",
            vec![
                (false, Some("entries")),
                (false, Some("records")),
                (false, Some("third")),
            ],
        ),
        (
            "WITH entries AS (SELECT id FROM public.entries) SELECT (SELECT e.* FROM entries e LIMIT 1)",
            vec![],
        ),
        (
            "WITH cte AS (SELECT id FROM entries) INSERT INTO records SELECT * FROM cte RETURNING *",
            vec![(true, Some("records"))],
        ),
    ];
    for (sql, expected) in cases {
        let stars = extract_fragment_stars(sql, None);
        let actual: Vec<_> = stars
            .iter()
            .map(|s| (s.returning, s.relation.as_deref()))
            .collect();
        assert_eq!(actual, expected, "sql: {sql}");
    }
}

#[test]
fn context_and_token_fallback_handle_prefixes_ctes_commas_and_subqueries() {
    let ctx = parse_context(
        "WITH RECURSIVE a(id, label) AS (SELECT 1, 'x'), b AS (SELECT 2) SELECT 1 FROM ONLY public.small s, wide AS w JOIN extra e ON e.id = w.id WHERE w.id = 1",
    );
    assert_eq!(ctx.ctes, vec!["a", "b"]);
    assert_eq!(
        ctx.from_relations,
        vec![
            ("public.small".to_string(), Some("s".to_string())),
            ("wide".to_string(), Some("w".to_string())),
            ("extra".to_string(), Some("e".to_string())),
        ]
    );

    let del_ctx = parse_context("DELETE FROM entries WHERE");
    assert_eq!(del_ctx.mutation_target, Some(("entries".to_string(), None)));

    let open_select = extract_fragment_stars("* FROM wide", Some("SELECT "));
    assert_eq!(open_select.len(), 1);
    assert_eq!(open_select[0].relation.as_deref(), Some("wide"));

    let open_returning = extract_fragment_stars("*", Some("UPDATE entries SET a = 1 RETURNING "));
    assert_eq!(open_returning.len(), 1);
    assert!(open_returning[0].returning);

    let comma_returning = extract_fragment_stars(", *", Some("UPDATE entries SET a = 1 RETURNING id"));
    assert_eq!(comma_returning.len(), 1);
    assert!(comma_returning[0].returning);

    let after_exists = extract_fragment_stars(
        "SELECT EXISTS (SELECT * FROM allowed_view), w.* FROM wide w WHERE",
        None,
    );
    assert_eq!(after_exists.len(), 1);
    assert_eq!(after_exists[0].relation.as_deref(), Some("wide"));

    let alias_over_cte = extract_fragment_stars(
        "WITH w AS (SELECT 1) SELECT w.* FROM wide AS w WHERE",
        None,
    );
    assert_eq!(alias_over_cte.len(), 1);
    assert_eq!(alias_over_cte[0].relation.as_deref(), Some("wide"));
}
