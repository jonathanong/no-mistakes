use super::*;

#[test]
fn fragment_local_legacy_lists_omit_the_normalized_triggers_segment() {
    for (fixture, key, line) in [
        ("locations-deprecated-list", "dependencies", 7),
        ("locations-shapes", "fullSuiteTriggers", 19),
    ] {
        let root = super::super::tests::fixture(fixture);
        let config = crate::config::v2::load_v2_config(&root, None).unwrap();
        let path = root.join(".no-mistakes.yml");
        let source = std::fs::read_to_string(&path).unwrap();
        let locations = ConfigLocations::parse(&source, &path).unwrap();
        let local = ConfigPath::default()
            .key("test_plan")
            .key("vitest")
            .key(key)
            .key("triggers")
            .index(0)
            .key("paths")
            .index(0);
        assert_eq!(locations.line(&local), None);
        assert_eq!(
            location_line(&locations, &local, &config),
            Some(line),
            "{fixture}"
        );
    }
}
