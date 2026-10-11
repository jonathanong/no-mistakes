use super::*;
use crate::codebase::postgres::statements::SqlFactSite;

fn saved_source(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn each_variant_retains_physical_positions_and_renumbered_bind_provenance() {
    let calls = calls("variants-placeholders.ts");
    let call = &calls[0];
    assert_eq!(call.variants.len(), 2);
    for version in &call.variants {
        assert_eq!(version.recovered_placeholder_positions.len(), 3);
        assert!(version.sql_text.contains("id > sql_placeholder_3"));
        assert!(version.sql_text.ends_with("sql_placeholder_1 = 0"));
        assert!(version
            .sql_source_positions
            .iter()
            .any(|position| matches!(position.source_line, 3 | 4)));
    }
}

#[test]
fn placeholder_digit_growth_preserves_branch_and_outer_physical_lines() {
    let file = file_facts("variants-placeholder-width.ts");
    let source = saved_source("variants-placeholder-width.ts");
    assert_eq!(file.calls[0].variants.len(), 2);
    for (index, variant) in file.calls[0].variants.iter().enumerate() {
        assert!(variant.sql_text.contains("WHERE id = sql_placeholder_10"));
        assert!(variant.sql_text.contains("AND owner = sql_placeholder_11"));
        assert_eq!(variant.recovered_placeholder_positions.len(), 11);
        let generated = &variant.recovered_placeholder_positions;
        let authored_column = variant
            .sql_text
            .rsplit('\n')
            .next()
            .unwrap()
            .rfind("sql_placeholder_10")
            .unwrap() as u32
            + 1;
        let authored_line = variant.sql_text.matches('\n').count() as u32 + 1;
        assert!(!generated.contains(&(authored_line, authored_column)));
        let call = file.calls[0].statement_calls().nth(index).unwrap();
        let tenth = variant.sql_text.find("sql_placeholder_10").unwrap();
        let outer = variant.sql_text.find("sql_placeholder_11").unwrap();
        assert_eq!(variant.sql_source_offsets.len(), variant.sql_text.len());
        let expression = ["${first}", "${second}"][index];
        assert_eq!(
            call.source_offset_at_sql_offset(tenth),
            Some(source.find(expression).unwrap() + 2)
        );
        assert_eq!(
            call.source_offset_at_sql_offset(outer),
            Some(source.find("${owner}").unwrap() + 2)
        );
        let authored = variant.sql_text.rfind("sql_placeholder_10").unwrap();
        assert_eq!(
            call.source_offset_at_sql_offset(authored),
            source.rfind("sql_placeholder_10")
        );
    }
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(facts[0].offset_uses[0].line, 4);
    assert_eq!(facts[1].limit_uses[0].line, 6);
}

#[test]
fn cooked_escapes_and_continuations_keep_branch_findings_on_physical_lines() {
    let file = file_facts("variants-escapes.ts");
    let source = saved_source("variants-escapes.ts");
    assert_eq!(
        texts(&file.calls[0]),
        [
            "SELECT id FROM users WHERE name = 'café'\n OFFSET 1",
            "SELECT id FROM users WHERE active\n LIMIT 1"
        ]
    );
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(facts[0].offset_uses[0].line, 4);
    assert_eq!(facts[1].limit_uses[0].line, 6);
    let call = file.calls[0].statement_calls().next().unwrap();
    let sql = call.sql_text.as_deref().unwrap();
    assert_eq!(
        call.source_offset_at_sql_offset(sql.find("OFFSET").unwrap()),
        source.find("OFFSET")
    );
    let unicode = sql.find('é').unwrap();
    assert_eq!(
        call.source_offset_at_sql_offset(unicode),
        source.find("\\u00e9")
    );
    assert_eq!(
        call.source_offset_at_sql_position(sql, 2, 2),
        source.find("OFFSET")
    );
}

#[test]
fn original_calls_use_their_existing_origins_when_no_mapping_entry_is_needed() {
    let file = file_facts("variants-single.ts");
    let calls = &file.calls;
    assert!(calls[0].sql_source_positions.is_empty());
    assert!(calls[3].sql_source_positions.is_empty());
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(facts[0].origin_line, 2);
    assert_eq!(facts[1].origin_line, 3);
    assert_eq!(facts[3].limit_uses[0].line, 9);
    assert_eq!(calls[0].source_offset_at_sql_offset(0), None);
}

#[test]
fn equivalent_sql_positions_keep_distinct_physical_branch_origins() {
    let name = "variants-optional-offset.ts";
    let file = file_facts(name);
    let source = saved_source(name);
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(file.calls[0].variants.len(), 2);
    for (index, call) in file.calls[0].statement_calls().enumerate() {
        let sql = call.sql_text.as_deref().unwrap();
        let sql_offset = sql.find("OFFSET").unwrap();
        let token = ["OFFSET 1", "OFFSET 2"][index];
        assert_eq!(
            call.source_offset_at_sql_position(sql, 1, sql_offset + 1),
            source.find(token)
        );
        assert_eq!(call.source_offset_at_sql_position(sql, 100, 1), None);
        assert_eq!(
            facts[index]
                .variant_locations
                .as_ref()
                .unwrap()
                .position(SqlFactSite::Offset(0))
                .unwrap()
                .source_offset,
            source.find(token)
        );
    }
}

#[test]
fn unicode_escapes_raw_templates_and_crlf_keep_exact_physical_byte_origins() {
    let name = "variants-origin-escapes.ts";
    let file = file_facts(name);
    let source = saved_source(name);
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(file.calls.len(), 4);
    for (index, call) in file.calls.iter().enumerate() {
        assert_eq!(call.variants.len(), 2, "{call:#?}");
        for (arm, variant) in call.statement_calls().enumerate() {
            let sql = variant.sql_text.as_deref().unwrap();
            let token = format!("OFFSET {}", 101 + index * 2 + arm);
            let offset = sql.find(&token).unwrap();
            let physical = source.find(&token).unwrap();
            assert_eq!(variant.source_offset_at_sql_offset(offset), Some(physical));
            let prepared = &facts[index * 2 + arm];
            assert_eq!(
                prepared.offset_uses[0].line,
                source[..physical]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1
            );
            let position = prepared
                .variant_locations
                .as_ref()
                .unwrap()
                .position(SqlFactSite::Offset(0))
                .unwrap();
            assert_eq!(position.source_offset, Some(physical));
            assert_eq!(call.variants[arm].sql_source_offsets.len(), sql.len());
        }
    }
    let call = file.calls[0].statement_calls().next().unwrap();
    let sql = call.sql_text.as_deref().unwrap();
    assert!(sql.contains("😀😀ABCé"));
    for (sql_offset, raw) in [
        (sql.find('😀').unwrap(), "\\u{1F600}"),
        (sql.rfind('😀').unwrap(), "\\uD83D\\uDE00"),
        (sql.find("ABC").unwrap(), "\\x41"),
        (sql.find('é').unwrap(), "é"),
    ] {
        assert_eq!(
            call.source_offset_at_sql_offset(sql_offset),
            source.find(raw)
        );
    }
    let raw = file.calls[3].statement_calls().next().unwrap();
    assert!(raw.sql_text.as_deref().unwrap().contains("\\u{41}\\n"));
}
