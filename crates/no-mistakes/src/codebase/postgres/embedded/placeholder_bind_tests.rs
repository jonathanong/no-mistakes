use super::placeholders::{
    count_placeholders, internal_bind, publish_placeholders, renumber_placeholders,
};

#[test]
fn appends_keep_a_single_generated_bind_sequence() {
    let head = format!("SELECT {}", internal_bind(1));
    let tail = format!(
        " WHERE a = {} AND b = {}",
        internal_bind(1),
        internal_bind(2)
    );
    let combined = format!(
        "{head}{}",
        renumber_placeholders(&tail, count_placeholders(&head))
    );
    assert_eq!(
        publish_placeholders(combined),
        "SELECT $1 WHERE a = $2 AND b = $3"
    );
}

#[test]
fn identifier_holes_do_not_advance_appended_value_bind_ordinals() {
    let head = format!(
        "SELECT {} FROM {}",
        internal_bind(1),
        super::placeholders::internal_placeholder(1)
    );
    let tail = format!(" WHERE owner = {}", internal_bind(1));
    let combined = format!(
        "{head}{}",
        renumber_placeholders(&tail, count_placeholders(&head))
    );
    assert_eq!(
        publish_placeholders(combined),
        "SELECT $1 FROM sql_placeholder_1 WHERE owner = $2"
    );
}

#[test]
fn value_rendering_preserves_relation_holes_and_function_argument_binds() {
    let sql = format!(
        "SELECT {} FROM {} JOIN generate_series(1, {}) series ON true WHERE id = {}",
        super::placeholders::internal_placeholder(1),
        super::placeholders::internal_placeholder(2),
        super::placeholders::internal_placeholder(3),
        super::placeholders::internal_placeholder(4)
    );
    let rendered = super::placeholders::parameterize_values(&sql);
    assert_eq!(
        publish_placeholders(rendered),
        "SELECT $1 FROM sql_placeholder_2 JOIN generate_series(1, $2) series ON true WHERE id = $3"
    );
}

#[test]
fn imported_scalar_projection_and_case_arms_stay_value_binds() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/parameterized-imported-values/query.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = super::extract_embedded_sql_from_source(
        &path,
        &source,
        &super::EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 5);
    for (call, expected) in facts.calls.iter().zip([
        "SELECT $1, $2 FROM entries OFFSET 1",
        "SELECT CASE WHEN true THEN $1 ELSE $2 END FROM entries OFFSET 1",
        "SELECT $1, CASE WHEN true THEN $2 ELSE $3 END FROM entries OFFSET 1",
        "SELECT $1, CASE WHEN true THEN $2 ELSE $3 END FROM entries OFFSET 1",
        "SELECT $1, CASE WHEN true THEN $2 ELSE $3 END FROM entries OFFSET 1",
    ]) {
        assert!(!call.is_unanalyzable(), "{call:#?}");
        assert_eq!(call.sql_text.as_deref(), Some(expected));
    }
}
