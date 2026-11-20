use super::specifier::{nested_pair, package_name};

#[test]
fn package_name_keeps_the_scope_and_drops_a_subpath() {
    assert_eq!(
        package_name("@widgets/ui/theme").as_deref(),
        Some("@widgets/ui")
    );
    assert_eq!(package_name("items/shape").as_deref(), Some("items"));
    assert_eq!(package_name("./local"), None);
    assert_eq!(package_name("node:fs"), None);
    assert_eq!(package_name("@widgets"), None);
    assert_eq!(package_name("@/ui"), None);
    assert_eq!(package_name("@widgets/"), None);
}

#[test]
fn nested_pair_accepts_only_one_separator() {
    assert_eq!(
        nested_pair(" widgets > items "),
        Some(("widgets".to_string(), "items".to_string()))
    );
    assert_eq!(nested_pair("react"), None);
    assert_eq!(nested_pair("widgets > items > extra"), None);
}
