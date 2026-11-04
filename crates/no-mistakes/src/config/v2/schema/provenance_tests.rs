use super::*;

#[test]
fn nearest_ancestor_preserves_key_and_index_suffixes() {
    let mut provenance = ConfigProvenance::default();
    let rules = ConfigPath::default().key("rules");
    let rule = rules.clone().index(3);
    let fragment = PathBuf::from(".no-mistakes/rules/types.yml");
    provenance.insert(
        rules,
        PathBuf::from(".no-mistakes.yml"),
        ConfigPath::default(),
    );
    provenance.insert(
        rule.clone(),
        fragment.clone(),
        ConfigPath::default().index(1),
    );
    let query = rule.key("options").key("roots").index(2);
    assert_eq!(
        provenance.origin(&query),
        Some((
            fragment.as_path(),
            ConfigPath::default()
                .index(1)
                .key("options")
                .key("roots")
                .index(2)
        ))
    );
    assert_eq!(
        provenance.origin(&ConfigPath::default().key("projects")),
        None
    );
}
