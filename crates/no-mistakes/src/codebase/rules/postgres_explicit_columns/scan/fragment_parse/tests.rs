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
        (
            "UPDATE narrow SET n = (SELECT count(*) FROM (SELECT * FROM wide) w) WHERE id IN (SELECT * FROM extra)",
            vec![(false, Some("wide")), (false, Some("extra"))],
        ),
        (
            "DELETE FROM narrow WHERE id IN (SELECT * FROM wide)",
            vec![(false, Some("wide"))],
        ),
        (
            "SELECT id FROM narrow ORDER BY (SELECT count(*) FROM (SELECT * FROM wide) w)",
            vec![(false, Some("wide"))],
        ),
        (
            "INSERT INTO narrow (id) VALUES ((SELECT count(*) FROM (SELECT * FROM wide) AS w))",
            vec![(false, Some("wide"))],
        ),
        (
            "SELECT j.* FROM (wide w JOIN narrow n ON w.id = n.id) AS j",
            vec![(false, Some("wide")), (false, Some("narrow"))],
        ),
        (
            "WITH RECURSIVE a AS (SELECT * FROM wide), wide AS (SELECT 1 AS id) SELECT id FROM a",
            vec![],
        ),
        (
            "WITH a AS (SELECT * FROM wide), wide AS (SELECT 1 AS id) SELECT id FROM a",
            vec![(false, Some("wide"))],
        ),
        (
            "INSERT INTO narrow (id, n) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET n = (SELECT count(*) FROM (SELECT * FROM wide) w) WHERE id IN (SELECT * FROM extra)",
            vec![(false, Some("wide")), (false, Some("extra"))],
        ),
        (
            "SELECT EXISTS (SELECT * FROM wide INTERSECT SELECT * FROM peer)",
            vec![(false, Some("wide")), (false, Some("peer"))],
        ),
        (
            r#"WITH "Wide" AS (SELECT 1 AS id) SELECT * FROM wide"#,
            vec![(false, Some("wide"))],
        ),
        (
            "SELECT pg_catalog.count(*), pg_catalog.count(w.*) FROM wide w",
            vec![(false, Some("wide"))],
        ),
        (
            "SELECT g.n FROM generate_series(1, (SELECT count(*) FROM (SELECT * FROM wide) w)) AS g(n)",
            vec![(false, Some("wide"))],
        ),
        (
            "SELECT (SELECT row_to_json(w.*)) FROM wide w",
            vec![(false, Some("wide"))],
        ),
        (
            "SELECT (SELECT w.* FROM narrow w) FROM wide w",
            vec![(false, Some("narrow"))],
        ),
        (
            "SELECT * FROM narrow LEFT JOIN wide w ON w.id = (SELECT * FROM extra LIMIT 1) LEFT OUTER JOIN l1 ON l1.id = narrow.id RIGHT JOIN a ON a.id = narrow.id RIGHT OUTER JOIN r1 ON r1.id = a.id FULL OUTER JOIN b ON b.id = a.id INNER JOIN c ON c.id = b.id CROSS JOIN d JOIN e USING (id)",
            vec![
                (false, Some("extra")),
                (false, Some("narrow")),
                (false, Some("wide")),
                (false, Some("l1")),
                (false, Some("a")),
                (false, Some("r1")),
                (false, Some("b")),
                (false, Some("c")),
                (false, Some("d")),
                (false, Some("e")),
            ],
        ),
        (
            "INSERT INTO wide AS w (id) VALUES (1) ON CONFLICT (id) DO NOTHING RETURNING w.*",
            vec![(true, Some("wide"))],
        ),
        (
            "INSERT INTO narrow (id, n) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET n = (SELECT * FROM wide LIMIT 1)",
            vec![(false, Some("wide"))],
        ),
        (
            "UPDATE wide SET a = 1",
            vec![],
        ),
        (
            "DELETE FROM wide",
            vec![],
        ),
        (
            "SELECT my_fn(*), fn(a => (SELECT * FROM wide)) FROM narrow WHERE array(SELECT * FROM extra) IS NOT NULL AND count(*) > 0 AND CURRENT_TIMESTAMP IS NOT NULL AND id IN UNNEST((SELECT * FROM u1)) AND id IN (1, (SELECT * FROM u2)) AND EXISTS (SELECT * FROM ignored)",
            vec![
                (false, Some("narrow")),
                (false, Some("wide")),
                (false, Some("extra")),
                (false, Some("u1")),
                (false, Some("u2")),
            ],
        ),
        (
            "SELECT wide.*, u.* FROM public.wide",
            vec![(false, Some("public.wide")), (false, None)],
        ),
        (
            "SELECT j.* FROM ((SELECT 1 AS id) d JOIN wide w ON true) AS j",
            vec![(false, Some("wide"))],
        ),
        (
            "WITH t AS (SELECT 1) TABLE t",
            vec![],
        ),
        (
            "TABLE wide",
            vec![(false, Some("wide"))],
        ),
        (
            "SELECT CASE id WHEN 1 THEN (SELECT * FROM a) ELSE (SELECT * FROM b) END, CAST((SELECT * FROM c) AS int), -(SELECT * FROM d), (SELECT * FROM e) IS NULL, (SELECT * FROM f) IS NOT NULL, (SELECT * FROM g) IS TRUE, (SELECT * FROM h) IS NOT TRUE, (SELECT * FROM i) IS FALSE, (SELECT * FROM j) IS NOT FALSE, (SELECT * FROM k) IS UNKNOWN, (SELECT * FROM l) IS NOT UNKNOWN, EXTRACT(YEAR FROM (SELECT * FROM m)), CEIL((SELECT * FROM n)), FLOOR((SELECT * FROM o)), id = ANY((SELECT * FROM p)), id = ALL((SELECT * FROM q)), name ILIKE (SELECT * FROM r), id IN UNNEST((SELECT * FROM s)), CURRENT_TIMESTAMP FROM narrow",
            vec![
                (false, Some("a")),
                (false, Some("b")),
                (false, Some("c")),
                (false, Some("d")),
                (false, Some("e")),
                (false, Some("f")),
                (false, Some("g")),
                (false, Some("h")),
                (false, Some("i")),
                (false, Some("j")),
                (false, Some("k")),
                (false, Some("l")),
                (false, Some("m")),
                (false, Some("n")),
                (false, Some("o")),
                (false, Some("p")),
                (false, Some("q")),
                (false, Some("r")),
                (false, Some("s")),
            ],
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

    let grouped = extract_fragment_stars(
        "SELECT count((w.*)), count((*)), row_to_json((w.*)), row_to_json((*)), row_to_json(CAST(w.* AS wide)) FROM wide w",
        None,
    );
    let grouped_actual: Vec<_> = grouped
        .iter()
        .map(|s| (s.relation.as_deref(), s.within_function.as_deref()))
        .collect();
    assert_eq!(
        grouped_actual,
        vec![
            (Some("wide"), Some("count")),
            (Some("wide"), Some("row_to_json")),
            (Some("wide"), Some("row_to_json")),
            (Some("wide"), Some("row_to_json")),
        ]
    );

    assert!(extract_fragment_stars(
        "SELECT (SELECT row_to_json(w.*) FROM generate_series(1, 3) AS w(n)) FROM wide AS w",
        None,
    )
    .is_empty());

    let filter_stars = extract_fragment_stars(
        "SELECT count(*) FILTER (WHERE (SELECT count(*) FROM (SELECT * FROM wide) w) > 0) FROM narrow WHERE count(*) FILTER (WHERE (SELECT count(*) FROM (SELECT * FROM extra) e) > 0) > 0",
        None,
    );
    let filter_rels: Vec<_> = filter_stars.iter().map(|s| s.relation.as_deref()).collect();
    assert_eq!(filter_rels, vec![Some("wide"), Some("extra")]);

    let nearer_scope = extract_fragment_stars(
        "SELECT (SELECT (SELECT row_to_json(wide.*)) FROM wide LIMIT 1) FROM narrow AS wide",
        None,
    );
    assert_eq!(nearer_scope.len(), 1);
    assert_eq!(nearer_scope[0].relation.as_deref(), Some("wide"));
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

    let nested_open = extract_fragment_stars("* FROM wide) q", Some("SELECT id FROM (SELECT "));
    assert_eq!(nested_open.len(), 1);
    assert_eq!(nested_open[0].relation.as_deref(), Some("wide"));

    let with_main_select = extract_fragment_stars(
        "WITH c AS (SELECT 1) SELECT *, wide AS payload FROM wide WHERE ",
        None,
    );
    assert_eq!(with_main_select.len(), 1);
    assert_eq!(with_main_select[0].relation.as_deref(), Some("wide"));

    let scoped_subquery = extract_fragment_stars(
        "SELECT (SELECT * FROM allowed_view LIMIT 1) FROM wide WHERE ",
        None,
    );
    assert_eq!(scoped_subquery.len(), 1);
    assert_eq!(scoped_subquery[0].relation.as_deref(), Some("allowed_view"));

    let multi_from = extract_fragment_stars(
        "SELECT * FROM narrow n JOIN wide w ON n.id = w.id WHERE ",
        None,
    );
    let multi_rels: Vec<_> = multi_from.iter().map(|s| s.relation.as_deref()).collect();
    assert_eq!(multi_rels, vec![Some("narrow"), Some("wide")]);

    let quoted_alias = extract_fragment_stars(
        r#"SELECT w.* FROM narrow AS "W", wide AS w WHERE"#,
        None,
    );
    assert_eq!(quoted_alias.len(), 1);
    assert_eq!(quoted_alias[0].relation.as_deref(), Some("wide"));

    let nested_in_exists = extract_fragment_stars(
        "SELECT EXISTS (SELECT 1 WHERE (1, 2) IN (SELECT * FROM wide)) WHERE",
        None,
    );
    assert_eq!(nested_in_exists.len(), 1);
    assert_eq!(nested_in_exists[0].relation.as_deref(), Some("wide"));

    let grouped_fn = extract_fragment_stars("SELECT row_to_json((w.*)) FROM wide w WHERE", None);
    assert_eq!(grouped_fn.len(), 1);
    assert_eq!(grouped_fn[0].within_function.as_deref(), Some("row_to_json"));

    let fn_in_subquery = extract_fragment_stars(
        "SELECT (SELECT count(v.*) FROM allowed_view v) FROM wide v WHERE",
        None,
    );
    assert_eq!(fn_in_subquery.len(), 1);
    assert_eq!(fn_in_subquery[0].relation.as_deref(), Some("allowed_view"));
    assert_eq!(fn_in_subquery[0].within_function.as_deref(), Some("count"));

    assert!(extract_fragment_stars(", w.*", Some("SELECT id FROM wide w ORDER BY id")).is_empty());

    let quoted_cte = extract_fragment_stars(
        r#"WITH "Wide" AS (SELECT 1) SELECT * FROM wide WHERE"#,
        None,
    );
    assert_eq!(quoted_cte.len(), 1);
    assert_eq!(quoted_cte[0].relation.as_deref(), Some("wide"));

    assert!(extract_fragment_stars("SELECT id FROM wide", None).is_empty());
    assert!(extract_fragment_stars("* FROM wide", Some("SELECT EXISTS (SELECT ")).is_empty());
    let intersect_stars = extract_fragment_stars(
        "* FROM wide INTERSECT SELECT * FROM peer",
        Some("SELECT EXISTS (SELECT "),
    );
    let intersect_rels: Vec<_> = intersect_stars
        .iter()
        .map(|s| s.relation.as_deref())
        .collect();
    assert_eq!(intersect_rels, vec![Some("wide"), Some("peer")]);

    let comma_sel = extract_fragment_stars(", w.* FROM wide w", Some("SELECT id"));
    assert_eq!(comma_sel.len(), 1);
    assert_eq!(comma_sel[0].relation.as_deref(), Some("wide"));

    let upsert_ctx = parse_context(
        "INSERT INTO wide (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = 1",
    );
    assert_eq!(upsert_ctx.mutation_target, Some(("wide".to_string(), None)));

    let ancestor_cte = extract_fragment_stars(
        "SELECT (WITH c AS (SELECT 1 AS id) SELECT (SELECT * FROM c LIMIT 1)) WHERE",
        None,
    );
    assert!(ancestor_cte.is_empty());

    let select_all = extract_fragment_stars("SELECT ALL * FROM wide WHERE", None);
    assert_eq!(select_all.len(), 1);
    assert_eq!(select_all[0].relation.as_deref(), Some("wide"));

    let distinct_on = extract_fragment_stars("SELECT DISTINCT ON (id) * FROM wide WHERE", None);
    assert_eq!(distinct_on.len(), 1);
    assert_eq!(distinct_on[0].relation.as_deref(), Some("wide"));

    assert!(extract_fragment_stars("SELECT (a + b) * c FROM wide WHERE", None).is_empty());

    let token_edges = extract_fragment_stars(
        "WITH c AS (SELECT 1) SELECT *, c.*, wide.*, u.* FROM public.wide, c WHERE",
        None,
    );
    let token_rels: Vec<_> = token_edges.iter().map(|s| s.relation.as_deref()).collect();
    assert_eq!(token_rels, vec![Some("public.wide"), Some("public.wide"), None]);

    let bare_no_from = extract_fragment_stars("SELECT * WHERE", None);
    assert_eq!(bare_no_from.len(), 1);
    assert_eq!(bare_no_from[0].relation, None);

    let ret_token = extract_fragment_stars("DELETE FROM wide RETURNING * WHERE", None);
    assert_eq!(ret_token.len(), 1);
    assert!(ret_token[0].returning);
    assert_eq!(ret_token[0].relation.as_deref(), Some("wide"));

    assert!(extract_fragment_stars(", w.*", Some("SELECT id FROM wide GROUP BY id")).is_empty());
    assert!(extract_fragment_stars(", w.*", Some("SELECT id FROM wide HAVING 1 = 1")).is_empty());
    assert!(extract_fragment_stars(", w.*", Some("SELECT id FROM wide LIMIT 1")).is_empty());
    assert!(extract_fragment_stars(", w.*", Some("SELECT id FROM wide UNION")).is_empty());
    assert!(extract_fragment_stars(", w.*", Some("SELECT id FROM wide INTERSECT")).is_empty());
    assert!(extract_fragment_stars(", w.*", Some("SELECT id FROM wide EXCEPT")).is_empty());
    assert!(extract_fragment_stars(", w.*", Some("UPDATE wide SET a = 1")).is_empty());
    assert!(extract_fragment_stars(", w.*", Some("INSERT INTO wide VALUES (1)")).is_empty());

    assert!(extract_fragment_stars("SELECT q.* FROM (SELECT id FROM wide) AS q WHERE", None).is_empty());

    let schema_fn = extract_fragment_stars("SELECT app.render(w.*) FROM wide w WHERE", None);
    assert_eq!(schema_fn.len(), 1);
    assert_eq!(schema_fn[0].within_function.as_deref(), Some("app.render"));

    let schema_qual = extract_fragment_stars("SELECT b.wide.* FROM a.wide, b.wide WHERE", None);
    assert_eq!(schema_qual.len(), 1);
    assert_eq!(schema_qual[0].relation.as_deref(), Some("b.wide"));

    assert_eq!(extract_fragment_stars("RETURNING * WHERE", None).len(), 1);
    assert_eq!(
        extract_fragment_stars(", * WHERE", Some("UPDATE entries SET a = 1 RETURNING id")).len(),
        1
    );
    assert_eq!(extract_fragment_stars(", * WHERE", Some("SELECT id")).len(), 1);
    assert_eq!(extract_fragment_stars("* FROM wide WHERE", Some("SELECT ")).len(), 1);
    assert_eq!(
        extract_fragment_stars("* WHERE", Some("UPDATE entries SET a = 1 RETURNING ")).len(),
        1
    );
    assert!(extract_fragment_stars("   ", None).is_empty());
    assert!(extract_fragment_stars("/* * */", None).is_empty());

    let table_line = extract_fragment_stars("SELECT 1;\nTABLE wide", None);
    assert_eq!(table_line.len(), 1);
    assert_eq!(table_line[0].line, 2);

    let paren_join = extract_fragment_stars(
        "SELECT j.* FROM (wide w JOIN narrow n ON w.id = n.id) AS j WHERE",
        None,
    );
    let paren_join_rels: Vec<_> = paren_join.iter().map(|s| s.relation.as_deref()).collect();
    assert_eq!(paren_join_rels, vec![Some("wide"), Some("narrow")]);

    let corr_tok = extract_fragment_stars(
        "SELECT (SELECT row_to_json(w.*)) FROM wide w WHERE",
        None,
    );
    assert_eq!(corr_tok.len(), 1);
    assert_eq!(corr_tok[0].relation.as_deref(), Some("wide"));

    assert!(extract_fragment_stars("SELECT pg_catalog.count(*) FROM wide WHERE", None).is_empty());

    let truncated_lit = extract_fragment_stars("SELECT * FROM wide WHERE note = '", None);
    assert_eq!(truncated_lit.len(), 1);
    assert_eq!(truncated_lit[0].relation.as_deref(), Some("wide"));

    let non_reserved_alias = extract_fragment_stars("SELECT value.* FROM wide value WHERE", None);
    assert_eq!(non_reserved_alias.len(), 1);
    assert_eq!(non_reserved_alias[0].relation.as_deref(), Some("wide"));

    let unmatched_paren = extract_fragment_stars("SELECT * FROM (", None);
    assert_eq!(unmatched_paren.len(), 1);
    assert_eq!(unmatched_paren[0].relation, None);

    let join_using = extract_fragment_stars("SELECT * FROM narrow JOIN wide USING (id) WHERE", None);
    let join_using_rels: Vec<_> = join_using.iter().map(|s| s.relation.as_deref()).collect();
    assert_eq!(join_using_rels, vec![Some("narrow"), Some("wide")]);

    assert!(extract_fragment_stars(
        "SELECT (SELECT q.* FROM (SELECT 1 AS id) q) FROM wide q WHERE",
        None,
    )
    .is_empty());

    assert!(extract_fragment_stars("SELECT * FROM (SELECT id FROM wide) q WHERE", None).is_empty());

    let edge_ctxs = [
        "WITH a(x, (y)) AS (SELECT 1), 1 SELECT 1",
        "WITH a(x",
        "WITH a SELECT 1",
        "SELECT 1 FROM ONLY",
        "SELECT 1 FROM public.",
        "SELECT 1 FROM wide AS",
        "SELECT 1 FROM wide",
        "UPDATE (1)",
        "DELETE FROM a USING b WHERE",
        "SELECT 1 FROM a, (SELECT 1) d JOIN (SELECT 2) e ON true, WHERE",
    ];
    for sql in edge_ctxs {
        let _ = parse_context(sql);
    }
}
