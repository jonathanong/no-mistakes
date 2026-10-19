use super::*;

#[test]
fn append_static_fragment_is_composed() {
    let facts = extract("composed-append.ts");
    assert_eq!(facts.calls[0].kind, super::super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}

#[test]
fn reassigned_let_is_dynamic() {
    let facts = extract("dynamic-let.ts");
    assert_eq!(facts.calls[0].kind, super::super::EmbeddedSqlKind::Dynamic);
}

#[test]
fn concat_fragments_and_non_const_inits_are_classified() {
    let nested = extract("composed-concat-nested.ts");
    assert_eq!(
        nested.calls[0].kind,
        super::super::EmbeddedSqlKind::Composed
    );
    assert_eq!(
        nested.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics")
    );
    assert_eq!(
        extract("composed-concat-templates.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Composed
    );
    assert_eq!(
        extract("composed-concat-tagged.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Composed
    );
    assert_eq!(
        extract("composed-concat-let.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        extract("composed-concat-dynamic.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        extract("composed-concat-minus.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        extract("let-uninitialized.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        extract("export-const-sql.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::ImmutableLocal
    );
    assert_eq!(
        extract("destructure-sql.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        extract("template-dynamic-init.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
}

#[test]
fn append_non_static_and_non_append_members_are_dynamic_or_unchanged() {
    for name in [
        "composed-append-spread.ts",
        "composed-append-dynamic-arg.ts",
        "composed-append-empty.ts",
    ] {
        assert_eq!(
            extract(name).calls[0].kind,
            super::super::EmbeddedSqlKind::Dynamic,
            "{name}"
        );
    }
    assert_eq!(
        extract("composed-append-other-method.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::ImmutableLocal
    );
    assert_eq!(
        extract("composed-append-non-ident.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::ImmutableLocal
    );
    assert_eq!(
        extract("composed-append-on-dynamic.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        extract("composed-append-function.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        extract("composed-append-unknown.ts").calls[0].kind,
        super::super::EmbeddedSqlKind::ImmutableLocal
    );
}

#[test]
fn inline_tagged_and_interpolated_calls_are_not_identifier_bindings() {
    // A free `sql` identifier is not a configured tag, so both forms stay dynamic.
    let tagged = extract_embedded_sql_from_source(
        Path::new("inline-tagged.ts"),
        "import { query } from '@example/db'\nquery(sql`SELECT 1`)\n",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(tagged.calls[0].kind, super::super::EmbeddedSqlKind::Dynamic);
    let interpolated = extract_embedded_sql_from_source(
        Path::new("inline-template.ts"),
        "import { query } from '@example/db'\nquery(`SELECT ${id}`)\n",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(
        interpolated.calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
    let raw = extract_embedded_sql_from_source(
        Path::new("inline-raw.ts"),
        "import { query } from '@example/db'\nquery(String.raw`SELECT ${table}`)\n",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(raw.calls[0].kind, super::super::EmbeddedSqlKind::Dynamic);
    let raw_static = extract_embedded_sql_from_source(
        Path::new("inline-raw-static.ts"),
        "import { query } from '@example/db'\nquery(String.raw`SELECT 1`)\n",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(
        raw_static.calls[0].kind,
        super::super::EmbeddedSqlKind::Inline
    );
    let raw_escaped = extract_embedded_sql_from_source(
        Path::new("inline-raw-escaped.ts"),
        "import { query } from '@example/db'\nquery(String.raw`SELECT 1\\nFROM topics`)\n",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(
        raw_escaped.calls[0].kind,
        super::super::EmbeddedSqlKind::Inline
    );
    assert_eq!(
        raw_escaped.calls[0].sql_text.as_deref(),
        Some("SELECT 1\\nFROM topics")
    );
    let sql_param = extract_embedded_sql_from_source(
        Path::new("inline-sql-param.ts"),
        "import { query } from '@example/db'\nquery(sql`SELECT ${id}`)\n",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(
        sql_param.calls[0].kind,
        super::super::EmbeddedSqlKind::Dynamic
    );
}

#[test]
fn unconfigured_executors_and_members_are_not_selected() {
    let path = fixture("executor-opt-in.ts");
    let source = read_fixture("executor-opt-in.ts");
    for (options, count) in [
        (EmbeddedSqlOptions::default(), 0),
        (EmbeddedSqlOptions::configured("", &["read".into()]), 1),
        (EmbeddedSqlOptions::configured("", &["query".into()]), 5),
        (EmbeddedSqlOptions::configured("@example/db", &[]), 2),
        (
            EmbeddedSqlOptions {
                import_specifier: "@example/db".into(),
                ..EmbeddedSqlOptions::default()
            },
            2,
        ),
        (
            EmbeddedSqlOptions::configured("@example/db", &["run".into()]),
            0,
        ),
    ] {
        let facts = extract_embedded_sql_from_source(&path, &source, &options);
        assert_eq!(facts.calls.len(), count, "{options:?}");
    }
}
