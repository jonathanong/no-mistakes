use super::{extract_set, fixture_root, SetSpec};
use std::collections::BTreeSet;

fn spec(strip_prefix: &str, exclude_prefix: &str) -> SetSpec {
    SetSpec {
        file: "pnpm-workspace.yaml".to_string(),
        kind: "yaml-sequence".to_string(),
        key: "minimumReleaseAgeExclude".to_string(),
        strip_prefix: strip_prefix.to_string(),
        exclude_prefix: exclude_prefix.to_string(),
        ..Default::default()
    }
}

fn set(strip_prefix: &str, exclude_prefix: &str) -> BTreeSet<String> {
    let root = fixture_root("fixture");
    extract_set(&root, &spec(strip_prefix, exclude_prefix), &[], &[])
        .unwrap()
        .values
}

#[test]
fn strip_prefix_strips_matching_values_and_drops_non_matching_ones() {
    assert_eq!(
        set("@acme/", ""),
        BTreeSet::from(["api".to_string(), "web".to_string(), "cli".to_string()]),
    );
    // "@acme/web" and "@acme/cli" do not carry the "@acme/a" prefix, so
    // stripPrefix drops them rather than leaving them unstripped.
    assert_eq!(set("@acme/a", ""), BTreeSet::from(["pi".to_string()]));
}

#[test]
fn exclude_prefix_drops_matching_values_and_keeps_the_rest() {
    assert_eq!(
        set("", "@acme/w"),
        BTreeSet::from(["@acme/api".to_string(), "@acme/cli".to_string()])
    );
    // Every member carries the prefix, so the whole set is excluded rather
    // than silently passing an empty-set comparison; pair with minSize: 1.
    assert!(set("", "@acme/").is_empty());
}

#[test]
fn strip_prefix_and_exclude_prefix_compose_in_that_order() {
    assert_eq!(
        set("@acme/", "c"),
        BTreeSet::from(["api".to_string(), "web".to_string()])
    );
}
