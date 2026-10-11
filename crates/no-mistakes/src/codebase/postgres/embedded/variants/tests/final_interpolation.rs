use super::*;

#[test]
fn final_interpolation_mutations_reject_stale_fragment_snapshots() {
    let calls = calls("variants-final-interpolation.ts");
    assert_eq!(calls.len(), 11);
    for call in &calls[..3] {
        assert!(call.is_unanalyzable(), "{call:#?}");
        assert!(call.variants.is_empty());
    }
    assert_eq!(
        texts(&calls[3]),
        [
            "SELECT id FROM documents WHERE id = 3 LIMIT sql_placeholder_1",
            "SELECT id FROM documents LIMIT sql_placeholder_1",
        ]
    );
    assert_eq!(
        texts(&calls[4]),
        [
            "SELECT id FROM fresh_users WHERE active AND enabled",
            "SELECT id FROM fresh_users WHERE active AND verified",
        ]
    );
    assert_eq!(
        texts(&calls[7]),
        ["SELECT id FROM single_users WHERE active LIMIT 1"]
    );
    assert_eq!(texts(&calls[8]), ["SELECT id FROM helper_users OFFSET 3"]);
    assert_eq!(calls, self::calls("variants-final-interpolation.ts"));
}

#[test]
fn primitive_capture_and_ordinary_final_binds_preserve_the_legacy_call_shape() {
    let calls = calls("variants-final-interpolation.ts");
    for index in [5, 6, 10] {
        assert_eq!(calls[index].kind, EmbeddedSqlKind::Inline);
        assert!(calls[index].variants.is_empty());
    }
    let mutated = &calls[5];
    let literal = &calls[6];
    assert_eq!(mutated.sql_text, literal.sql_text);
    assert_eq!(
        mutated.recovered_placeholder_positions,
        literal.recovered_placeholder_positions
    );
    assert_eq!(
        mutated.sql_text.as_deref(),
        Some("SELECT sql_placeholder_1 LIMIT sql_placeholder_2")
    );
    assert_eq!(
        calls[10].sql_text.as_deref(),
        Some("SELECT sql_placeholder_1, sql_placeholder_2 LIMIT sql_placeholder_3")
    );
}

#[test]
fn final_bind_renumbering_preserves_branch_fragment_and_outer_byte_origins() {
    let file = file_facts("variants-final-interpolation.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let call = &file.calls[9];
    assert_eq!(call.variants.len(), 2);
    let branches = [("${branchFirst}", 3), ("${branchSecond}", 2)];
    for (variant, (branch, count)) in call.variants.iter().zip(branches) {
        assert_eq!(variant.recovered_placeholder_positions.len(), count);
        assert_eq!(
            variant.sql_source_offsets[variant.sql_text.find("sql_placeholder_1").unwrap()]
                as usize,
            source.find(branch).unwrap() + 2
        );
        let last = format!("sql_placeholder_{count}");
        assert_eq!(
            variant.sql_source_offsets[variant.sql_text.find(&last).unwrap()] as usize,
            source.find("${limit}").unwrap() + 2
        );
    }
    let version = call.statement_calls().next().unwrap();
    let sql = version.sql_text.as_deref().unwrap();
    assert_eq!(
        version.source_offset_at_sql_offset(sql.find("OFFSET").unwrap()),
        source.rfind("OFFSET")
    );
}
