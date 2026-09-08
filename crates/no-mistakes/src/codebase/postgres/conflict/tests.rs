use super::*;

#[test]
fn preserves_expression_conflict_targets_while_parsing_the_source_order() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO categories (item_id, category) SELECT item_id, category FROM input ORDER BY item_id, lower(category) ON CONFLICT (item_id, (LOWER(category))) DO NOTHING",
    )
    .unwrap();
    assert_eq!(inserts.len(), 1);
    assert_eq!(
        inserts[0].target,
        SqlConflictTarget::Columns {
            expressions: vec!["item_id".to_string(), "(LOWER(category))".to_string()],
            predicate: None,
        }
    );
    assert!(inserts[0].source.multi_row);
    assert_eq!(inserts[0].source.order.as_ref().unwrap().len(), 2);
}

#[test]
fn marks_targetless_conflicts() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM input ORDER BY id ON CONFLICT DO NOTHING",
    )
    .unwrap();
    assert_eq!(inserts[0].target, SqlConflictTarget::Targetless);
}

#[test]
fn preserves_partial_index_predicates() {
    let inserts = analyze_conflict_inserts(
        "INSERT INTO items (id) SELECT id FROM input ORDER BY id ON CONFLICT (id) WHERE is_live DO NOTHING",
    )
    .unwrap();
    assert_eq!(
        inserts[0].target,
        SqlConflictTarget::Columns {
            expressions: vec!["id".to_string()],
            predicate: Some("is_live".to_string()),
        }
    );
}
