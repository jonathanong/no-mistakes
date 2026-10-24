use super::{ConfigLocations, ConfigPath, Segment};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/source-locations")
        .join(name)
}

fn locations(name: &str) -> ConfigLocations {
    let path = fixture(name);
    ConfigLocations::parse(&std::fs::read_to_string(&path).unwrap(), &path).unwrap()
}

fn project() -> ConfigPath {
    ConfigPath::default().key("projects").key("web")
}

#[test]
fn typed_paths_do_not_confuse_key_punctuation_with_structure() {
    let path = ConfigPath::default()
        .key("quoted.key")
        .key("with[brackets]")
        .index(2);
    assert_eq!(
        path.as_slice(),
        &[
            Segment::Key("quoted.key".into()),
            Segment::Key("with[brackets]".into()),
            Segment::Index(2),
        ]
    );
    assert_eq!(path, path.clone());
}

#[test]
fn block_yaml_tracks_keys_items_and_multiline_scalars() {
    let locations = locations("block.yaml");
    assert_eq!(locations.line(&project().key("root")), Some(4));
    assert_eq!(locations.line(&project().key("include")), Some(5));
    assert_eq!(locations.line(&project().key("include").index(0)), Some(6));
    assert_eq!(locations.line(&project().key("include").index(1)), Some(7));
    assert_eq!(locations.line(&project().key("after")), Some(11));
    assert_eq!(
        locations.line(&project().key("multiline").key("root")),
        None
    );
    assert_eq!(
        locations.line(
            &ConfigPath::default()
                .key("quoted.key")
                .key("with[brackets]")
        ),
        Some(13)
    );
    assert_eq!(
        locations.line(&ConfigPath::default().key("é").key("it's")),
        Some(15)
    );
    assert_eq!(
        locations.line(&ConfigPath::default().key("scalar")),
        Some(16)
    );
    assert_eq!(
        locations.line(&ConfigPath::default().key("scalar").index(0)),
        Some(17)
    );
}

#[test]
fn flow_yaml_and_unknown_paths_are_supported() {
    let locations = locations("flow.yml");
    assert_eq!(locations.line(&project().key("include").index(1)), Some(2));
    assert_eq!(
        locations.line(
            &ConfigPath::default()
                .key("quoted.key")
                .key("with[brackets]")
        ),
        Some(3)
    );
    assert_eq!(locations.line(&project().key("absent")), None);
    assert_eq!(locations.line(&project().key("include").index(2)), None);
    assert_eq!(locations.line(&project().index(0)), None);
}

#[test]
fn aliases_preserve_reference_lines_without_expanding_merges_or_cycles() {
    let locations = locations("aliases.yaml");
    assert_eq!(locations.line(&project().key("root")), Some(7));
    assert_eq!(locations.line(&project().key("include").index(0)), Some(7));
    let api = ConfigPath::default().key("projects").key("api");
    assert_eq!(locations.line(&api.clone().key("root")), Some(10));
    // serde_yaml does not implicitly apply YAML merge keys to typed config fields.
    assert_eq!(locations.line(&api.clone().key("include")), None);
    assert_eq!(locations.line(&api.key("<<").key("root")), Some(9));
    assert_eq!(
        locations.line(&ConfigPath::default().key("other").index(1)),
        Some(12)
    );
    let recursive = ConfigPath::default().key("recursive").key("self");
    assert_eq!(locations.line(&recursive.clone().key("self")), Some(14));
    assert_eq!(locations.line(&recursive.key("absent")), None);
    assert_eq!(
        locations.line(&ConfigPath::default().key("byAliasKey").key("root")),
        Some(17)
    );
}

#[test]
fn json_tracks_decoded_keys_property_lines_and_array_elements() {
    let locations = locations("nested.json");
    assert_eq!(locations.line(&project().key("root")), Some(4));
    assert_eq!(locations.line(&project().key("include").index(1)), Some(7));
    assert_eq!(
        locations.line(
            &ConfigPath::default()
                .key("quoted.key")
                .key("with[brackets]")
        ),
        Some(11)
    );
    assert_eq!(
        locations.line(&ConfigPath::default().key("é").key("it's")),
        Some(12)
    );
    assert_eq!(
        locations.line(&ConfigPath::default().key("scalar")),
        Some(13)
    );
    assert_eq!(
        locations.line(&ConfigPath::default().key("scalar").index(0)),
        Some(14)
    );
    assert_eq!(
        locations.line(&ConfigPath::default().key("duplicate")),
        Some(16)
    );
}

#[test]
fn jsonc_retains_existing_comment_and_trailing_comma_support() {
    let locations = locations("comments.jsonc");
    assert_eq!(locations.line(&project()), Some(4));
    assert_eq!(locations.line(&project().key("include").index(1)), Some(7));
}

#[test]
fn yaml_duplicate_keys_select_the_last_authored_location() {
    assert_eq!(
        locations("duplicate.yaml").line(&ConfigPath::default().key("root")),
        Some(3)
    );
}

#[test]
fn metadata_failure_is_optional_and_does_not_widen_the_parser_contract() {
    for name in [
        "invalid.yaml",
        "invalid.json",
        "invalid.jsonc",
        "json5.jsonc",
        "multiple.yaml",
        "unknown.toml",
        "tagged-scalar.yaml",
        "tagged-mapping.yaml",
        "tagged-sequence.yaml",
        "empty.json",
    ] {
        let path = fixture(name);
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(ConfigLocations::parse(&source, &path).is_none(), "{name}");
    }
    let path = fixture("empty.yaml");
    let source = std::fs::read_to_string(&path).unwrap();
    if let Some(locations) = ConfigLocations::parse(&source, &path) {
        assert_eq!(locations.line(&ConfigPath::default().key("root")), None);
    }
}

#[test]
fn wide_maps_keep_winning_property_lines_and_alias_key_identity() {
    let locations = locations("wide-projects.yaml");
    let projects = ConfigPath::default().key("projects");
    for index in 0..128 {
        let project = projects.clone().key(format!("p{index:03}"));
        let key_line = if index == 64 { 261 } else { 5 + 2 * index };
        assert_eq!(locations.line(&project), Some(key_line));
        assert_eq!(locations.line(&project.key("root")), Some(key_line + 1));
    }
    assert_eq!(
        locations.line(&projects.clone().key("byAliasKey").key("root")),
        Some(264)
    );
    assert_eq!(
        locations.line(&projects.clone().key("byAlias").key("root")),
        Some(265)
    );
    assert_eq!(
        locations.line(&projects.key("escaped.key[0]").key("root")),
        Some(267)
    );
    assert_eq!(locations.line(&ConfigPath::default().key("not")), None);
}

#[test]
fn long_comment_blocks_do_not_displace_collection_entry_locations() {
    let locations = locations("comment-block.yaml");
    assert_eq!(locations.line(&project().key("root")), Some(131));
}
