use super::ParsedProgramCache;
use std::sync::Arc;

#[test]
fn eviction_releases_the_interned_path_and_every_mode_only_for_that_file() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-facts/bounded-session-memory");
    let first = root.join("source.ts");
    let second = root.join("outer.ts");
    let cache = ParsedProgramCache::default();
    for path in [&first, &second] {
        let text: Arc<str> = std::fs::read_to_string(path).unwrap().into();
        cache.with_program(path, text.clone(), |_, _| ()).unwrap();
        cache
            .with_recovered_typescript_program_observed(path, text, || {}, |_, _, _| ())
            .unwrap();
    }
    assert_eq!(cache.entry_count(), 4);
    assert_eq!(cache.interned_paths.borrow().len(), 2);
    cache.remove_path(&root.join("./source.ts"));
    assert_eq!(cache.entry_count(), 2);
    assert!(cache.contains_path(&second));
    assert!(!cache.contains_path(&first));
    assert_eq!(cache.interned_paths.borrow().len(), 1);
    cache.remove_path(&first);
    cache.remove_path(&second);
    assert_eq!(cache.entry_count(), 0);
    assert!(cache.interned_paths.borrow().is_empty());
}
