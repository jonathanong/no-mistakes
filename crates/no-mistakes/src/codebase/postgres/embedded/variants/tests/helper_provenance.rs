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
