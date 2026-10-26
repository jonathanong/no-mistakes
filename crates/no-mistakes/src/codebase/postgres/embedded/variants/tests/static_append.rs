use super::*;

#[test]
fn deterministic_trusted_appends_keep_a_parseable_singleton_and_physical_binds() {
    let file = file_facts("variants-static-append.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(file.calls.len(), 11);
    for call in &file.calls[..5] {
        assert_eq!(call.variants.len(), 1, "{call:#?}");
        // Legacy raw classification can stop before a long fluent chain; the
        // recovered singleton still proves the exact supplied statement.
        assert!(!call.is_unanalyzable());
        let statement = call.statement_calls().next().unwrap();
        assert_eq!(statement.kind, EmbeddedSqlKind::Composed);
        assert_eq!(
            statement.sql_text.as_deref(),
            Some(call.variants[0].sql_text.as_str())
        );
        if let Some(raw) = &call.sql_text {
            assert_eq!(&call.variants[0].sql_text, raw);
        }
        assert!(!call.variants[0].append_sites.is_empty());
        assert!(!call.variants_truncated());
    }
    let direct = &file.calls[0].variants[0];
    assert_eq!(direct.sql_text, "SELECT id FROM users WHERE owner = sql_placeholder_1 AND active AND team = sql_placeholder_2");
    assert_eq!(direct.recovered_placeholder_positions.len(), 2);
    for (marker, origin) in [
        ("sql_placeholder_1", "${owner}"),
        ("sql_placeholder_2", "${team}"),
    ] {
        assert_eq!(
            direct.sql_source_offsets[direct.sql_text.find(marker).unwrap()] as usize,
            source.find(origin).unwrap() + 2
        );
    }
    // Nested fluent calls share their receiver start offset; occurrence IDs
    // deduplicate that span, so assert every suffix survives in execution order.
    assert_eq!(
        file.calls[3].variants[0].sql_text,
        format!(
            "SELECT 4{}",
            (0..20)
                .map(|index| format!(" /* link{index} */"))
                .collect::<String>()
        )
    );
    let parsed = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, false, true);
    assert_eq!(parsed.len(), 5);
    assert_eq!(
        parsed
            .iter()
            .map(|facts| facts.parse_failed)
            .collect::<Vec<_>>(),
        [false, false, false, false, true]
    );
}

#[test]
fn singleton_publication_does_not_trust_string_methods_or_revoked_builders() {
    let calls = calls("variants-static-append.ts");
    for call in &calls[5..] {
        assert!(call.variants.is_empty(), "{call:#?}");
    }
    // Escapes and later arguments must revoke the earlier first-argument proof.
    assert!(calls[9].is_unanalyzable());
    assert!(calls[10].is_unanalyzable());
}
