use super::*;

#[test]
fn final_identifier_rebinding_preserves_captured_fragments_and_replacement_reads() {
    let calls = calls("variants-final-rebinding.ts");
    assert_eq!(calls.len(), 17);
    assert_eq!(
        texts(&calls[0]),
        [
            "SELECT id FROM users WHERE id = sql_placeholder_1 OFFSET sql_placeholder_2 LIMIT sql_placeholder_3",
            "SELECT id FROM users WHERE id = sql_placeholder_1 LIMIT sql_placeholder_2",
        ]
    );
    assert_eq!(texts(&calls[1]), ["SELECT id FROM replacements OFFSET 9"]);
    for (index, first) in [(2, 1), (3, 3), (4, 5), (5, 7), (8, 9)] {
        let expected = [
            format!("SELECT {first} LIMIT sql_placeholder_1"),
            format!("SELECT {} LIMIT sql_placeholder_1", first + 1),
        ];
        assert_eq!(texts(&calls[index]), expected, "call {index}");
    }
    assert_eq!(
        texts(&calls[6]),
        [
            "SELECT id FROM aliases OFFSET 2 LIMIT sql_placeholder_1",
            "SELECT id FROM aliases LIMIT sql_placeholder_1",
        ]
    );
    assert_eq!(
        texts(&calls[7]),
        ["SELECT id FROM aliases OFFSET 2", "SELECT id FROM aliases"]
    );
    assert_eq!(texts(&calls[16]), ["SELECT id FROM replacements OFFSET 9"]);
    assert_eq!(calls, self::calls("variants-final-rebinding.ts"));
}

#[test]
fn final_rebinding_still_rejects_rhs_mutations_escapes_and_earlier_assignments() {
    let calls = calls("variants-final-rebinding.ts");
    for (index, call) in calls[9..=15].iter().enumerate() {
        let index = index + 9;
        assert!(call.is_unanalyzable(), "call {index}: {:?}", call);
        assert!(call.variants.is_empty(), "call {index}");
    }
}

#[test]
fn final_rebinding_keeps_captured_branch_and_final_bind_byte_origins() {
    let file = file_facts("variants-final-rebinding.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let call = &file.calls[0];
    let last_origin = source.find("${(fragment = replacement, limit)}").unwrap() + 2;
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
            last_origin
        );
    }
    let projected = call.statement_calls().next().unwrap();
    let sql = projected.sql_text.as_deref().unwrap();
    assert_eq!(
        projected.source_offset_at_sql_offset(sql.find("OFFSET").unwrap()),
        source.find("OFFSET")
    );
}
