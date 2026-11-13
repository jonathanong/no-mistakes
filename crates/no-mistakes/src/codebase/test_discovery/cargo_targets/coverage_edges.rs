use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::codebase::ts_resolver::normalize_path;

fn manifest(name: &str) -> toml::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/coverage-region-edges/cargo")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    toml::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

#[test]
fn manifest_parse_coverage_edges() {
    let directory = PathBuf::from("/repo/pkg");
    let visible = BTreeSet::new();
    assert!(
        super::manifest::parse(&directory, &manifest("missing-name.toml"), &visible, None)
            .is_none()
    );
    assert!(
        super::manifest::parse(&directory, &manifest("numeric-name.toml"), &visible, None)
            .is_none()
    );

    let entry = normalize_path(&directory.join("tests/widget.rs"));
    let visible = BTreeSet::from([entry.as_path()]);
    let package =
        super::manifest::parse(&directory, &manifest("widget.toml"), &visible, Some("2021"))
            .unwrap();
    assert!(package.targets.iter().any(|target| {
        target.name == "widget" && target.flag == "--test" && target.entry == entry
    }));
}
