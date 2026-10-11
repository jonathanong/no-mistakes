use super::*;

#[test]
fn final_assignment_result_consumers_cannot_publish_stale_captured_builders() {
    let calls = calls("variants-final-assignment-consumers.ts");
    assert_eq!(calls.len(), 9);
    for (index, call) in calls[..6].iter().enumerate() {
        assert!(call.is_unanalyzable(), "call {index}: {call:#?}");
        assert!(call.variants.is_empty(), "call {index}: {call:#?}");
    }
    assert_eq!(calls[8].kind, EmbeddedSqlKind::Inline);
    assert!(calls[8].variants.is_empty());
    assert_eq!(
        calls[8].sql_text.as_deref(),
        Some("SELECT sql_placeholder_1 LIMIT sql_placeholder_2")
    );
    assert_eq!(calls, self::calls("variants-final-assignment-consumers.ts"));
}

#[test]
fn discarded_assignment_values_keep_fragment_origins_and_later_alias_reads() {
    let file = file_facts("variants-final-assignment-consumers.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(
        texts(&file.calls[6]),
        [
            "SELECT id FROM unchanged_users WHERE id = sql_placeholder_1 OFFSET sql_placeholder_2 LIMIT sql_placeholder_3",
            "SELECT id FROM unchanged_users WHERE id = sql_placeholder_1 LIMIT sql_placeholder_2",
        ]
    );
    assert_eq!(
        texts(&file.calls[7]),
        [
            "SELECT id FROM unchanged_users WHERE id = sql_placeholder_1 OFFSET sql_placeholder_2",
            "SELECT id FROM unchanged_users WHERE id = sql_placeholder_1",
        ]
    );
    let call = &file.calls[6];
    for (variant, (branch, count)) in call
        .variants
        .iter()
        .zip([("${firstOwner}", 3), ("${secondOwner}", 2)])
    {
        assert_eq!(variant.recovered_placeholder_positions.len(), count);
        assert_eq!(
            variant.sql_source_offsets[variant.sql_text.find("sql_placeholder_1").unwrap()]
                as usize,
            source.find(branch).unwrap() + 2
        );
        assert_eq!(
            variant.sql_source_offsets[variant
                .sql_text
                .find(&format!("sql_placeholder_{count}"))
                .unwrap()] as usize,
            source.find("${(other = unchanged, limit)}").unwrap() + 2
        );
    }
    let projected = call.statement_calls().next().unwrap();
    let sql = projected.sql_text.as_deref().unwrap();
    assert_eq!(
        projected.source_offset_at_sql_offset(sql.find("OFFSET").unwrap()),
        source.rfind("OFFSET")
    );
}
