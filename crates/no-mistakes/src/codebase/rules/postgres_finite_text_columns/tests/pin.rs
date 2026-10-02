fn pins(sql: &str, column: &str) -> Option<Vec<String>> {
    let expr = super::super::parse::expression(sql)?;
    super::super::pin::pinned_literals_expr(&expr, column)
}

fn eq(sql: &str, column: &str, expected: Option<&[&str]>) {
    let expected = expected.map(|values| values.iter().map(|value| (*value).to_string()).collect());
    assert_eq!(pins(sql, column), expected, "{sql}");
}

#[test]
fn equality_in_any_and_null_pin_literals() {
    eq("status = 'draft'", "status", Some(&["draft"]));
    eq("'sent' = status", "status", Some(&["sent"]));
    eq("kind IN ('a', 'b')", "kind", Some(&["a", "b"]));
    eq(
        "status = ANY (ARRAY['draft'::text, 'sent'::text, 'paid'::text])",
        "status",
        Some(&["draft", "sent", "paid"]),
    );
    eq("kind IS NULL", "kind", Some(&[]));
    eq("(kind)::text = 'open'::text", "kind", Some(&["open"]));
    eq(
        "kind::character varying = 'open'::character varying",
        "kind",
        Some(&["open"]),
    );
    eq(r#""Kind" = 'special'"#, "Kind", Some(&["special"]));
    eq("STATUS = 'draft'", "status", Some(&["draft"]));
    eq("status IN ('open', lower('CLOSED'))", "status", None);
    eq("status::char(1) = 'x'::text", "status", None);
    eq("status::varchar = 'open'::text", "status", Some(&["open"]));
    eq(
        "status = ANY ((ARRAY['open', 'closed']::text[]))",
        "status",
        Some(&["open", "closed"]),
    );
}

#[test]
fn or_requires_every_branch_and_and_keeps_the_first_list() {
    eq(
        "(kind = 'a') OR (kind = 'b') OR (kind IS NULL)",
        "kind",
        Some(&["a", "b"]),
    );
    eq("(kind = 'a') OR (score > 1)", "kind", None);
    eq("(score > 1) AND (kind = 'b')", "kind", Some(&["b"]));
    eq(
        "status IN ('a', 'b') AND status IN ('b', 'c')",
        "status",
        Some(&["a", "b"]),
    );
    eq("(kind IS NULL) AND (kind IS NULL)", "kind", Some(&[]));
    eq("(((kind = 'a')))", "kind", Some(&["a"]));
    eq(
        "((scope = 'global') AND (tenant_id IS NULL)) OR ((scope = 'tenant') AND (tenant_id IS NOT NULL))",
        "scope",
        Some(&["global", "tenant"]),
    );
    eq(
        "((scope = 'global') AND (tenant_id IS NULL)) OR ((scope = 'tenant') AND (tenant_id IS NOT NULL))",
        "tenant_id",
        None,
    );
}

#[test]
fn negated_forms_other_columns_and_functions_do_not_pin() {
    eq("status <> 'deleted'", "status", None);
    eq("status NOT IN ('deleted')", "status", None);
    eq("status <> ALL (ARRAY['deleted'::text])", "status", None);
    eq("NOT (status = 'deleted')", "status", None);
    eq("status LIKE 'a%'", "status", None);
    eq("status ~ '^a'", "status", None);
    eq("status = other_status", "status", None);
    eq(
        "status = ANY (ARRAY[other_status, 'x'::text])",
        "status",
        None,
    );
    eq("char_length(label) <= 100", "label", None);
    eq("score = ANY (ARRAY['-1'::integer, 0, 1])", "score", None);
    eq(
        "((kind <> 'refund') OR (reason_code = 'fraud'))",
        "kind",
        None,
    );
    eq(
        "((kind <> 'refund') OR (reason_code = 'fraud'))",
        "reason_code",
        None,
    );
    eq("this is not sql !!!", "status", None);
    eq("status = 'draft', trailing", "status", Some(&["draft"]));
    eq("(')", "status", None);
    eq("status = ANY (other_status)", "status", None);
    eq("status = E'draft'", "status", Some(&["draft"]));
    eq("status = N'draft'", "status", Some(&["draft"]));
    eq("status = $$draft$$", "status", Some(&["draft"]));
}
