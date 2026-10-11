use super::*;

#[test]
fn final_effects_match_captured_identities_and_uncertain_alias_groups() {
    let file = file_facts("variants-captured-builder-identities.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let rows = source
        .lines()
        .filter(|line| line.starts_with("query("))
        .collect::<Vec<_>>();
    assert_eq!(file.calls.len(), rows.len());
    assert_eq!(rows.len(), 67);
    for (call, row) in file.calls.iter().zip(rows) {
        if row.contains("opaque") {
            assert!(call.is_unanalyzable(), "{row}: {call:#?}");
            assert!(call.variants.is_empty());
        } else if let Some((_, expected)) = row.split_once("variants: ") {
            assert_eq!(
                texts(call),
                expected.split(" | ").collect::<Vec<_>>(),
                "{row}"
            );
        } else {
            assert!(row.contains("inline: "));
            assert_eq!(call.kind, EmbeddedSqlKind::Inline);
            assert!(call.variants.is_empty());
            assert_eq!(
                call.sql_text.as_deref(),
                row.split_once("inline: ").map(|(_, sql)| sql)
            );
        }
    }
    assert_eq!(file.calls, calls("variants-captured-builder-identities.ts"));
}

#[test]
fn independent_final_effects_keep_placeholder_and_physical_branch_provenance() {
    let file = file_facts("variants-captured-builder-identities.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let call = &file.calls[17];
    assert_eq!(call.variants.len(), 2);
    for (variant, (first, count)) in call
        .variants
        .iter()
        .zip([("${branchFirst}", 3), ("${branchSecond}", 2)])
    {
        assert_eq!(variant.recovered_placeholder_positions.len(), count);
        assert_eq!(
            variant.sql_source_offsets[variant.sql_text.find("sql_placeholder_1").unwrap()]
                as usize,
            source.find(first).unwrap() + 2
        );
        let last = format!("sql_placeholder_{count}");
        let sql_offset = variant.sql_text.find(&last).unwrap();
        assert_eq!(
            variant.sql_source_offsets[sql_offset] as usize,
            source.find("${(locationOther").unwrap() + 2
        );
    }
    let version = call.statement_calls().next().unwrap();
    let offset = version.sql_text.as_deref().unwrap().find("OFFSET").unwrap();
    assert_eq!(
        version.source_offset_at_sql_offset(offset),
        source.find("OFFSET ${skip}")
    );
    let primitive = &file.calls[15];
    let literal = &file.calls[16];
    assert_eq!(primitive.kind, literal.kind);
    assert_eq!(primitive.sql_text, literal.sql_text);
    assert_eq!(
        primitive.recovered_placeholder_positions,
        literal.recovered_placeholder_positions
    );
}
