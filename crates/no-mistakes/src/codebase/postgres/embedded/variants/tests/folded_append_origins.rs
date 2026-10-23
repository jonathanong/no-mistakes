use super::*;

#[test]
fn folded_append_choice_keeps_bind_origins_at_direct_and_spliced_executors() {
    let file = file_facts("variants-folded-append-provenance.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    let sites = source
        .match_indices("statement.append(suffix)")
        .map(|(offset, _)| offset as u32)
        .collect::<Vec<_>>();
    let bind_origin = source.find("${id}").unwrap() + 2;
    assert_eq!(file.calls.len(), 2);
    for call in &file.calls {
        assert_eq!(call.variants.len(), 1);
        let variant = &call.variants[0];
        assert_eq!(
            variant.sql_text,
            "SELECT id FROM accounts WHERE id = sql_placeholder_1"
        );
        assert_eq!(variant.recovered_placeholder_positions.len(), 1);
        let bind = variant.sql_text.find("sql_placeholder_1").unwrap();
        assert_eq!(variant.sql_source_offsets[bind] as usize, bind_origin);
        let mut actual = variant.append_sites.clone();
        actual.sort_unstable();
        assert_eq!(actual, sites);
    }
}
