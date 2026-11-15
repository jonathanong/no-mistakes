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
    ] {
        assert_ne!(
            super::tokens(left),
            super::tokens(right),
            "{left} vs {right}"
        );
        assert!(super::tokens(left).is_some(), "{left}");
    }
}
