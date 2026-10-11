use super::*;

#[test]
fn conditional_helper_appends_retain_each_literal_sql_origin() {
    let file = file_facts("variants-helper-provenance.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(file.calls.len(), 1);
    let call = &file.calls[0];
    assert_eq!(call.variants.len(), 2, "{call:#?}");
    for version in &call.variants {
        for (sql_offset, _) in version.sql_text.match_indices("SELECT *") {
            let offset = version.sql_source_offsets[sql_offset] as usize;
            assert!(source[offset..].starts_with("SELECT *"));
        }
    }
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(facts.len(), 2);
    for facts in facts {
        assert!(!facts.parse_failed);
        let lines: Vec<_> = facts
            .selects
            .iter()
            .flat_map(|select| select.star_projections.iter().map(|star| star.line))
            .collect();
        assert_eq!(lines, [14, 21]);
    }
}

#[test]
fn conditional_helper_renumbered_bind_preserves_later_star_origins() {
    let file = file_facts("variants-helper-placeholder-provenance.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(file.calls.len(), 1);
    let call = &file.calls[0];
    assert_eq!(call.variants.len(), 2, "{call:#?}");
    let first = source.rfind("SELECT *").unwrap();
    let second = source.find("SELECT *").unwrap();
    let bound = source.find("${values[9]}").unwrap() + 2;
    for version in &call.variants {
        assert_eq!(version.sql_source_offsets.len(), version.sql_text.len());
        let stars: Vec<_> = version
            .sql_text
            .match_indices("SELECT *")
            .map(|(sql_offset, _)| version.sql_source_offsets[sql_offset] as usize)
            .collect();
        assert_eq!(stars, [first, second]);
        let has_tenth_bind = version.sql_text.contains("sql_placeholder_10");
        assert_eq!(
            version.recovered_placeholder_positions.len(),
            if has_tenth_bind { 10 } else { 9 }
        );
        if has_tenth_bind {
            let offset = version.sql_text.find("sql_placeholder_10").unwrap();
            assert_eq!(version.sql_source_offsets[offset] as usize, bound);
        }
    }
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts(&file, true, false);
    assert_eq!(facts.len(), 2);
    for facts in facts {
        assert!(!facts.parse_failed);
        let lines: Vec<_> = facts
            .selects
            .iter()
            .flat_map(|select| select.star_projections.iter().map(|star| star.line))
            .collect();
        assert_eq!(lines, [21, 11]);
    }
}
