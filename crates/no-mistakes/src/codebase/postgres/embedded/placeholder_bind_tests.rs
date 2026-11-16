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
