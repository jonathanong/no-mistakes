use super::{extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn extract(name: &str) -> super::EmbeddedSqlFileFacts {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture");
    extract_embedded_sql_from_source(&fixture(name), &source, &EmbeddedSqlOptions::default())
}

#[test]
fn function_local_sql_template_append_is_composed() {
    let facts = extract("composed-append-function-local.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(facts.calls[0].callee, "read");
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE published = true ORDER BY id LIMIT 50")
    );
}

#[test]
fn function_local_write_append_is_composed() {
    let facts = extract("composed-append-function-local-write.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(facts.calls[0].callee, "write");
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some(
            "UPDATE topics SET title = sql_placeholder_1 WHERE id = sql_placeholder_2 AND published = true"
        )
    );
}

#[test]
fn append_of_a_bound_helper_sql_statement_is_composed() {
    let facts = extract("composed-append-helper-ident.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some(
            "SELECT id FROM topics WHERE published = true AND tenant_id = sql_placeholder_1 ORDER BY id"
        )
    );
}

#[test]
fn append_of_a_helper_sql_statement_call_is_composed() {
    let facts = extract("composed-append-helper-call.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE published = true AND tenant_id = sql_placeholder_1")
    );
}

#[test]
fn function_local_conditional_static_append_is_composed() {
    let facts = extract("composed-append-if-function-local.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE published = true ORDER BY id")
    );
}

#[test]
fn unrecoverable_helper_append_fails_closed() {
    let facts = extract("composed-append-unrecoverable-helper.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[0].sql_text, None);
}

#[test]
fn unbound_append_argument_fails_closed() {
    let facts = extract("composed-append-unbound-ident.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(facts.calls[0].sql_text, None);
}

#[test]
fn append_inside_unbraced_if_is_composed() {
    let facts = extract("composed-append-if.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics WHERE id = 1")
    );
}

#[test]
fn append_inside_unbraced_loop_is_dynamic() {
    let facts = extract("composed-append-loop.ts");
    assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn append_inside_switch_ternary_or_logical_is_composed() {
    for name in [
        "composed-append-switch.ts",
        "composed-append-ternary.ts",
        "composed-append-and.ts",
    ] {
        let facts = extract(name);
        assert_eq!(facts.calls[0].kind, EmbeddedSqlKind::Composed, "{name}");
        assert_eq!(
            facts.calls[0].sql_text.as_deref(),
            Some("SELECT id FROM topics WHERE id = 1"),
            "{name}"
        );
    }
}

#[test]
fn append_inside_loop_forms_is_dynamic() {
    for name in [
        "composed-append-for.ts",
        "composed-append-for-in.ts",
        "composed-append-while.ts",
        "composed-append-do-while.ts",
    ] {
        assert_eq!(
            extract(name).calls[0].kind,
            EmbeddedSqlKind::Dynamic,
            "{name}"
        );
    }
}
