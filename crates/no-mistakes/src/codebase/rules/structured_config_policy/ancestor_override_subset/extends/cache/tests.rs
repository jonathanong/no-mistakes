use super::*;

#[test]
fn returns_the_cached_value_for_repeated_parses() {
    let path = Path::new("config.yml");
    let mut cache = ParsedAncestorCache::default();

    let first = cache.parse(path, "rules: {}").unwrap();
    let second = cache
        .parse(path, "rules: invalid-but-not-reparsed")
        .unwrap();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(cache.values.len(), 1);
}
