use super::*;

#[test]
fn folded_and_nested_variants_retain_contributing_append_occurrences() {
    let file = file_facts("variants-append-sites.ts");
    let source = std::fs::read_to_string(&file.path).unwrap();
    assert_eq!(file.calls.len(), 3);
    let sites: Vec<u32> = source
        .match_indices("q.append(common)")
        .map(|(offset, _)| offset as u32)
        .collect();
    assert_eq!(sites.len(), 2);
    for call in &file.calls[..2] {
        assert_eq!(call.variants.len(), 1);
        let mut actual = call.variants[0].append_sites.clone();
        actual.sort_unstable();
        assert_eq!(actual, sites);
    }
    let fluent = source
        .find("sql`SELECT id FROM accounts`.append(common)")
        .unwrap() as u32;
    assert!(file.calls[2]
        .variants
        .iter()
        .any(|variant| variant.append_sites == [fluent]));
    assert!(file.calls[2]
        .variants
        .iter()
        .any(|variant| variant.append_sites.is_empty()));
    assert!(file.fragment_sites.contains(&Some(fluent)));
}
