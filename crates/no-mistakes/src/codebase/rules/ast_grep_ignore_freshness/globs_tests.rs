use super::matches_tracked;

#[test]
fn dot_slash_prefix_does_not_match_the_tracked_path() {
    let tracked = vec!["src/items.ts".to_string()];
    assert_eq!(
        matches_tracked("./src/items.ts", false, "", &tracked),
        Ok(false)
    );
    assert_eq!(
        matches_tracked("src/items.ts", false, "", &tracked),
        Ok(true)
    );
}

#[test]
fn nested_project_uses_only_paths_under_the_config_directory() {
    let tracked = vec![
        "src/orders.ts".to_string(),
        "packages/widgets/src/items.ts".to_string(),
    ];
    assert_eq!(
        matches_tracked("src/items.ts", false, "packages/widgets", &tracked),
        Ok(true)
    );
    assert_eq!(
        matches_tracked("src/orders.ts", false, "packages/widgets", &tracked),
        Ok(false)
    );
}

#[test]
fn bang_prefix_is_a_literal_character() {
    let tracked = vec!["src/items.ts".to_string()];
    assert_eq!(
        matches_tracked("!src/items.ts", false, "", &tracked),
        Ok(false)
    );
}
