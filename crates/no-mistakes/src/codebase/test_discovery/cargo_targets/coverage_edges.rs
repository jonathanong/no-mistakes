use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::codebase::ts_resolver::normalize_path;

#[test]
fn manifest_parse_coverage_edges() {
    let directory = PathBuf::from("/repo/pkg");
    let visible = BTreeSet::new();
    let missing: toml::Value = toml::from_str("[package]\nversion = \"0.1.0\"\n").unwrap();
    assert!(super::manifest::parse(&directory, &missing, &visible, None).is_none());

    let number: toml::Value = toml::from_str("[package]\nname = 1\n").unwrap();
    assert!(super::manifest::parse(&directory, &number, &visible, None).is_none());

    let named: toml::Value = toml::from_str("[package]\nname = \"widget\"\n").unwrap();
    let entry = normalize_path(&directory.join("tests/widget.rs"));
    let visible = BTreeSet::from([entry.as_path()]);
    let package = super::manifest::parse(&directory, &named, &visible, Some("2021")).unwrap();
    assert!(package.targets.iter().any(|target| {
        target.name == "widget" && target.flag == "--test" && target.entry == entry
    }));
}
