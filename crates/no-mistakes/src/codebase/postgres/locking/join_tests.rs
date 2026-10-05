use super::{
    extract_locking_select_metadata, extract_locking_select_metadata_with_placeholders,
    JoinEquality, LockingSelectMetadata,
};

fn only(sql: &str) -> LockingSelectMetadata {
    let mut locks = extract_locking_select_metadata(sql).unwrap();
    assert_eq!(locks.len(), 1, "{sql}");
    locks.remove(0)
}

fn join(left: (&str, &str), right: (&str, &str)) -> JoinEquality {
    JoinEquality {
        left_table: left.0.to_string(),
        left_column: left.1.to_string(),
        right_table: right.0.to_string(),
        right_column: right.1.to_string(),
    }
}

#[test]
fn inner_join_and_where_equalities_between_relations_are_join_facts() {
    let sql = "SELECT 1 FROM codes c JOIN grants g ON g.id = c.grant_id AND g.x > 1 \
               INNER JOIN clients k ON (k.id = g.client_id) \
               WHERE c.token = $1 AND k.owner = c.owner FOR UPDATE OF c, g";
    let meta = only(sql);
    assert_eq!(
        meta.join_equalities,
        [
            join(("clients", "owner"), ("codes", "owner")),
            join(("grants", "id"), ("codes", "grant_id")),
            join(("clients", "id"), ("grants", "client_id")),
        ]
    );
    assert_eq!(meta.pinned_columns.unwrap()["codes"], ["token"]);
}

#[test]
fn outer_or_nested_and_self_equalities_are_not_join_facts() {
    for from_where in [
        "codes c LEFT JOIN grants g ON g.id = c.grant_id WHERE c.t = $1",
        "codes c RIGHT JOIN grants g ON g.id = c.grant_id WHERE c.t = $1",
        "codes c JOIN grants g ON g.id = c.grant_id OR g.x = c.x WHERE c.t = $1",
        "codes c JOIN grants g ON NOT (g.id = c.grant_id) WHERE c.t = $1",
        "codes c JOIN grants g ON g.id = g.parent WHERE c.t = $1",
        "codes c JOIN grants g ON g.id = lower(c.grant_id) WHERE c.t = $1",
        "codes c JOIN grants g ON true JOIN grants h ON g.id = c.grant_id WHERE c.t = $1",
        "codes c CROSS JOIN grants g WHERE c.t = $1 AND id = c.grant_id",
        "codes c JOIN LATERAL (SELECT 1) d ON d.x = c.x WHERE c.t = $1",
    ] {
        let meta = only(&format!("SELECT 1 FROM {from_where} FOR UPDATE OF c"));
        assert!(meta.join_equalities.is_empty(), "{from_where}");
    }
}

#[test]
fn recovered_interpolations_are_binds_only_at_their_recorded_positions() {
    let sql = "SELECT 1 FROM orders WHERE id = sql_placeholder_0 AND region = sql_placeholder_1 \
               FOR UPDATE";
    let at = |positions: &[(u32, u32)]| {
        extract_locking_select_metadata_with_placeholders(sql, positions)
            .unwrap()
            .remove(0)
            .pinned_columns
            .unwrap()
            .remove("orders")
            .unwrap()
    };
    // Column 33 is the first marker; the second marker's position is not recorded.
    assert_eq!(at(&[(1, 33)]), ["id"]);
    assert!(at(&[]).is_empty());
    // The no-positions entry point never treats a marker spelling as a bind.
    assert!(only(sql).pinned_columns.unwrap()["orders"].is_empty());
}
