use super::*;

#[test]
fn absent_dependabot_updates_and_cooldown_have_distinct_meanings() {
    let root = fixture("cooldown-shapes");
    let paths = [root.join("no-updates.yml"), root.join("no-cooldown.yml")];
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    assert!(inputs::cooldown(&sources, &paths[0]).unwrap().is_none());
    // An npm update with no cooldown is present, with an empty exclusion list.
    assert!(inputs::cooldown(&sources, &paths[1])
        .unwrap()
        .unwrap()
        .is_empty());
    assert_eq!(sources.physical_read_count(), 2);
}
