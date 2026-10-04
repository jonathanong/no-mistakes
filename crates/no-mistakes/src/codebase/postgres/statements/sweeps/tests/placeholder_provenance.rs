use super::{
    extract_sql_statement_facts, extract_sql_statement_facts_with_recovered_placeholders,
    shape_sweeps,
};

fn recovered_shape(sql: &str) -> Vec<String> {
    let positions = sql
        .match_indices("sql_placeholder_")
        .map(|(offset, _)| {
            let prefix = &sql[..offset];
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1;
            let column = prefix
                .rsplit('\n')
                .next()
                .map_or(0, |line| line.chars().count()) as u32
                + 1;
            (line, column)
        })
        .collect::<Vec<_>>();
    let facts = extract_sql_statement_facts_with_recovered_placeholders(sql, true, &positions);
    shape_sweeps(&facts.sweeps)
}

#[test]
fn recovered_interpolations_are_binds() {
    // `${after}` and `${size}` reach the facts as sql_placeholder_N identifiers.
    assert_eq!(
        recovered_shape("SELECT id FROM orders WHERE id > sql_placeholder_1 ORDER BY id LIMIT sql_placeholder_2"),
        ["orders id | id > sql_placeholder_1(id)"]
    );
    assert_eq!(
        recovered_shape("SELECT 1 FROM t WHERE (sql_placeholder_1::uuid IS NULL OR id > sql_placeholder_1) ORDER BY id LIMIT 3"),
        ["t id | sql_placeholder_1::uuid is null or id > sql_placeholder_1(id)"]
    );
    assert_eq!(
        recovered_shape("SELECT 1 FROM t WHERE (a, b) > (sql_placeholder_1, sql_placeholder_2) ORDER BY a, b LIMIT 3"),
        ["t a,b | (a, b) > (sql_placeholder_1, sql_placeholder_2)(a,b)"]
    );
    // A bind is never the column side of a cursor.
    assert_eq!(
        recovered_shape(
            "SELECT 1 FROM t WHERE sql_placeholder_1 > sql_placeholder_2 ORDER BY id LIMIT 3"
        ),
        ["t id | sql_placeholder_1 > sql_placeholder_2()"]
    );
}

#[test]
fn public_extractor_treats_marker_shaped_identifiers_as_columns() {
    let facts = extract_sql_statement_facts(
        "SELECT id FROM orders WHERE sql_placeholder_1 = $1 AND id > $2 ORDER BY id LIMIT $3",
    );
    assert!(!facts.sweeps[0].conjuncts[0].bind_guard);
    assert!(facts.sweeps[0].conjuncts[0].cursor_columns.is_empty());
}

#[test]
fn bind_only_guards_do_not_select_rows() {
    let facts = super::sweeps("SELECT id FROM orders WHERE $1::boolean IS NOT NULL AND id > $2 AND deleted_at IS NULL AND FALSE AND now() > $3 ORDER BY id LIMIT $4");
    assert_eq!(
        facts[0]
            .conjuncts
            .iter()
            .map(|fact| fact.bind_guard)
            .collect::<Vec<_>>(),
        vec![true, false, false, false, false]
    );
}
