#[test]
fn equivalent_cast_spellings_share_expression_index_tokens() {
    for (left, right) in [
        ("CAST(id AS text)", "id::text"),
        ("cast(id as TEXT)", "id::text"),
        ("lower(CAST(id AS text))", "lower(id::text)"),
        ("(CAST(id AS text))", "id::text"),
        ("CAST(id AS text) COLLATE \"C\"", "id::text COLLATE \"C\""),
        ("CAST(a || b AS text)", "(a || b)::text"),
        ("CAST(id AS text) || label", "id::text || label"),
        ("CAST(CAST(id AS text) AS uuid)", "id::text::uuid"),
        ("CAST(id AS text)::uuid", "id::text::uuid"),
        ("CAST(ROW(a, b) AS item_identity)", "(a, b)::item_identity"),
        (
            "CAST(ROW(a, b) AS item_identity)",
            "ROW(a, b)::item_identity",
        ),
        ("CAST(ROW(a) AS item_identity)", "ROW(a)::item_identity"),
        ("CAST(id AS double precision)", "id::double precision"),
        (
            "CAST(id AS timestamp with time zone)",
            "id::timestamp with time zone",
        ),
        ("CAST(id AS varchar(32))", "id::varchar(32)"),
        ("CAST(id AS text[])", "id::text[]"),
        ("CAST(id AS pg_catalog.text)", "id::pg_catalog.text"),
        ("CAST(\"ID\" AS text)", "\"ID\"::text"),
        ("CAST(-id AS text)", "(-id)::text"),
        ("CAST(f(a, b) AS text)", "f(a, b)::text"),
        ("CAST(a.b AS text)", "a.b::text"),
        (
            "lower(CAST(id::text || label AS text))",
            "lower((id::text || label)::text)",
        ),
        (
            "CAST(id::double precision AS text)",
            "id::double precision::text",
        ),
        ("CAST(id::double AS text)", "id::double::text"),
        (
            "CAST(id::character varying(32) AS text)",
            "id::character varying(32)::text",
        ),
        ("CAST(id::character(8) AS text)", "id::character(8)::text"),
        ("CAST(id::bit varying AS text)", "id::bit varying::text"),
        (
            "CAST(id::national character varying(2) AS text)",
            "id::national character varying(2)::text",
        ),
        ("CAST(id::national AS text)", "id::national::text"),
        (
            "CAST(id::timestamp with time zone AS text)",
            "id::timestamp with time zone::text",
        ),
        (
            "CAST(id::time(3) without time zone AS text)",
            "id::time(3) without time zone::text",
        ),
        ("CAST(id::timestamp AS text)", "id::timestamp::text"),
        (
            "CAST(id::pg_catalog.text AS uuid)",
            "id::pg_catalog.text::uuid",
        ),
        (
            "CAST(id::pg_catalog.text(4)[] AS uuid)",
            "id::pg_catalog.text(4)[]::uuid",
        ),
        ("CAST(id::text[] AS uuid)", "id::text[]::uuid"),
        ("CAST(id::varchar(32) AS text)", "id::varchar(32)::text"),
        (
            "CAST(id::char varying(36) AS text)",
            "id::char varying(36)::text",
        ),
        ("CAST(\"not\"(x) AS boolean)", "\"not\"(x)::boolean"),
        (
            "CAST(id::timestamp with instant AS text)",
            "(id::timestamp with instant)::text",
        ),
        ("CAST(id::schema.+1 AS text)", "(id::schema.+1)::text"),
        ("CAST(id:: AS text)", "(id::)::text"),
        ("CAST(id::(int) AS text)", "(id::(int))::text"),
        ("CAST(N'abc' AS text)", "N'abc'::text"),
        ("CAST(X'DEAD' AS text)", "X'DEAD'::text"),
        ("CAST(EXISTS (SELECT 1) AS text)", "EXISTS (SELECT 1)::text"),
        ("CAST(ARRAY[1] AS text)", "ARRAY[1]::text"),
        ("CAST(ARRAY(SELECT 1) AS text)", "ARRAY(SELECT 1)::text"),
        ("CAST(ARRAY 1 AS text)", "(ARRAY 1)::text"),
        (
            "CAST(CASE WHEN 1 THEN CASE WHEN 0 THEN 2 END END AS text)",
            "CASE WHEN 1 THEN CASE WHEN 0 THEN 2 END END::text",
        ),
        (
            "CAST(CASE WHEN 1 THEN 2 AS text)",
            "(CASE WHEN 1 THEN 2)::text",
        ),
        ("CAST(id[ AS text)", "(id[)::text"),
        ("CAST(f(g(a)) AS text)", "f(g(a))::text"),
        ("CAST(a.+1 AS text)", "(a.+1)::text"),
        ("CAST(EXISTS 1 AS text)", "(EXISTS 1)::text"),
    ] {
        assert_eq!(
            super::tokens(left),
            super::tokens(right),
            "{left} vs {right}"
        );
    }
}

#[test]
fn distinct_cast_spellings_keep_expression_index_tokens() {
    for (left, right) in [
        ("CAST(id AS uuid)", "id::text"),
        ("CAST(id AS \"text\")", "id::text"),
        ("CAST(\"ID\" AS text)", "id::text"),
        ("CAST(id AS text) COLLATE \"C\"", "id::text"),
        (
            "CAST(id AS text) COLLATE \"C\"",
            "CAST(id AS text) COLLATE \"POSIX\"",
        ),
        ("CAST(a || b AS text)", "a || b::text"),
        ("TRY_CAST(id AS text)", "id::text"),
        ("SAFE_CAST(id AS text)", "id::text"),
        ("pg_catalog.cast(id AS text)", "id::text"),
        ("CAST(ROW(a) AS item_identity)", "(a)::item_identity"),
        ("CAST(-id AS text)", "-id::text"),
        ("CAST((id) AS text)", "id::text"),
        ("CAST(id AS text)::text", "id::text"),
        (
            "CAST(id::timestamp with time zone AS text)",
            "(id::timestamp with time zone)::text",
        ),
        (
            "CAST(id::double precision AS text)",
            "(id::double precision)::text",
        ),
        (
            "CAST(id::timestamp with instant AS text)",
            "id::timestamp with instant::text",
        ),
        ("CAST id AS text", "id::text"),
        ("CAST(id AS text AS uuid)", "id::text::uuid"),
        ("CAST(id AS text", "id::text"),
        ("CAST(NOT (x) AS boolean)", "NOT (x)::boolean"),
        ("CAST(ANY (x) AS boolean)", "ANY (x)::boolean"),
        ("CAST(ALL (x) AS boolean)", "ALL (x)::boolean"),
        ("CAST(SOME (x) AS boolean)", "SOME (x)::boolean"),
    ] {
        assert_ne!(
            super::tokens(left),
            super::tokens(right),
            "{left} vs {right}"
        );
        assert!(super::tokens(left).is_some(), "{left}");
    }
}
