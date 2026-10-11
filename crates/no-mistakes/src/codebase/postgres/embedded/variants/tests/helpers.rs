use super::*;
use crate::codebase::postgres::statements::SqlFactSite;

#[test]
fn direct_parameter_helpers_keep_append_origins_and_renumbered_bind_positions() {
    let file = file_facts("variants-parameter-helpers.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(file.calls.len(), 14);
    let call = &file.calls[0];
    assert_eq!(call.variants.len(), 2);
    for (variant, marker, count) in [
        (&call.variants[0], "sql_placeholder_10", 10),
        (&call.variants[1], "sql_placeholder_2", 2),
    ] {
        assert_eq!(variant.recovered_placeholder_positions.len(), count);
        let offset = variant.sql_text.find(marker).unwrap();
        assert_eq!(
            variant.sql_source_offsets[offset] as usize,
            source.find("${owner}").unwrap() + 2
        );
    }
    let prepared = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert!(prepared.len() >= 7);
    let offset_origin = source.find("OFFSET 7").unwrap();
    // LIMIT sites preserve the legacy count-expression position; OFFSET sites
    // deliberately point at the keyword.
    let limit_origin = source.find("LIMIT 3").unwrap() + "LIMIT ".len();
    for facts in prepared
        .iter()
        .filter(|facts| !facts.offset_uses.is_empty())
    {
        assert!(!facts.parse_failed);
        let locations = facts.variant_locations.as_ref().unwrap();
        for (site, offset) in [
            (SqlFactSite::Offset(0), offset_origin),
            (SqlFactSite::Limit(0), limit_origin),
        ] {
            let position = locations.position(site).unwrap();
            assert_eq!(position.source_offset, Some(offset));
            assert_eq!(
                position.source_line,
                source[..offset]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1
            );
        }
    }
    assert_eq!(
        texts(&file.calls[1]),
        ["SELECT id FROM users WHERE owner_id = sql_placeholder_1 OFFSET 7 LIMIT 3"]
    );
    assert_eq!(
        texts(&file.calls[2]),
        [
            "SELECT id FROM users WHERE owner_id = sql_placeholder_1 OFFSET 7 LIMIT 3 FOR UPDATE",
            "SELECT 2"
        ]
    );
    assert_eq!(
        texts(&file.calls[3]),
        [
            "SELECT id FROM users WHERE owner_id = sql_placeholder_1 OFFSET 7 LIMIT 3",
            "SELECT 9"
        ]
    );
    assert!(file.calls[4..].iter().all(EmbeddedSqlCall::is_unanalyzable));
}

#[test]
fn stored_parameter_helper_returns_never_reuse_a_stale_builder_snapshot() {
    let calls = calls("variants-stored-helpers.ts");
    assert_eq!(calls.len(), 13);
    assert!(
        calls.iter().all(EmbeddedSqlCall::is_unanalyzable),
        "{calls:#?}"
    );
}

#[test]
fn static_helper_append_forms_keep_cooked_and_raw_physical_origins() {
    let file = file_facts("variants-helper-appends.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(file.calls.len(), 1);
    assert_eq!(file.calls[0].variants.len(), 2);
    let version = &file.calls[0].variants[0];
    assert_eq!(
        version.sql_text,
        r"SELECT id FROM users WHERE active = true AND name = E'\\n' LIMIT 5"
    );
    for (sql_token, source_token) in [
        ("FROM", "FROM"),
        ("WHERE", "WHERE"),
        ("AND", "AND"),
        ("LIMIT", "LIMIT"),
        ("ers", r"\u0065rs"),
        (r"\\n", r"\\n"),
    ] {
        assert_eq!(
            version.sql_source_offsets[version.sql_text.find(sql_token).unwrap()] as usize,
            source.find(source_token).unwrap()
        );
    }
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert!(facts.iter().all(|facts| !facts.parse_failed));
    let position = facts[0]
        .variant_locations
        .as_ref()
        .unwrap()
        .position(SqlFactSite::Limit(0))
        .unwrap();
    assert_eq!(
        position.source_offset,
        source.find("LIMIT 5").map(|offset| offset + "LIMIT ".len())
    );
}

#[test]
fn typed_helper_detection_tracks_import_aliases_and_returned_fragment_values() {
    let calls = calls("variants-helper-detection.ts");
    assert_eq!(calls.len(), 5);
    assert_eq!(texts(&calls[0]), ["SELECT id FROM users"]);
    assert!(calls[1].is_unanalyzable());
    for call in &calls[2..] {
        assert_eq!(call.kind, EmbeddedSqlKind::Inline);
        assert_eq!(call.sql_text.as_deref(), Some("SELECT sql_placeholder_1"));
        assert!(call.variants.is_empty());
    }
}

#[test]
fn parameter_helpers_reject_unsupported_bodies_and_inputs_without_inventing_sql() {
    let calls = calls("variants-helper-rejections.ts");
    assert_eq!(calls.len(), 20);
    for (index, call) in calls.iter().enumerate().take(19) {
        if index == 16 {
            continue;
        }
        assert!(call.is_unanalyzable(), "call {index}: {call:#?}");
    }
    // Direct helper calls preserve the legacy static-string input recovery;
    // the finite-variant helper guard remains stricter for branch arms.
    assert_eq!(calls[16].kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        calls[16].sql_text.as_deref(),
        Some("SELECT id FROM users WHERE owner = sql_placeholder_1")
    );
    assert!(calls[16].variants.is_empty());
    assert_eq!(texts(&calls[19]), ["SELECT id FROM users", "SELECT 17"]);
}
