use super::*;
use std::fs;
use tempfile::tempdir;

#[derive(Default, serde::Deserialize)]
struct TestConfig {
    name: String,
}

#[test]
fn test_load_config_yaml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test.yaml");
    fs::write(&config_path, "name: hello\n").unwrap();

    let config: TestConfig = load_config(dir.path(), None, &["test"]).unwrap();
    assert_eq!(config.name, "hello");
}

#[test]
fn test_load_config_json() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test.json");
    fs::write(&config_path, "{\"name\": \"world\"}").unwrap();

    let config: TestConfig = load_config(dir.path(), None, &["test"]).unwrap();
    assert_eq!(config.name, "world");
}

#[test]
fn test_load_config_priority() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("first.yaml"), "name: first\n").unwrap();
    fs::write(dir.path().join("second.yaml"), "name: second\n").unwrap();

    let config: TestConfig = load_config(dir.path(), None, &["first", "second"]).unwrap();
    assert_eq!(config.name, "first");
}

#[test]
fn test_load_config_missing_returns_default() {
    let dir = tempdir().unwrap();
    let config: TestConfig = load_config(dir.path(), None, &["missing"]).unwrap();
    assert_eq!(config.name, "");
}

#[test]
fn test_load_config_explicit() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("custom.yaml");
    fs::write(&config_path, "name: custom\n").unwrap();

    let config: TestConfig =
        load_config(dir.path(), Some(Path::new("custom.yaml")), &["test"]).unwrap();
    assert_eq!(config.name, "custom");
}

#[test]
fn pass5a_legacy_visible_loader_matches_discovery_and_honors_explicit_ignored_config() {
    let dir = crate::test_support::materialize_gitignore_fixture("auto-discovery");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(dir.path());

    let discovered: TestConfig = load_config(dir.path(), None, &["legacy-visible"]).unwrap();
    let prepared: TestConfig =
        load_config_from_visible(dir.path(), None, &["legacy-visible"], &visible).unwrap();
    assert_eq!(prepared.name, discovered.name);
    assert_eq!(prepared.name, "visible");

    let ignored: TestConfig =
        load_config_from_visible(dir.path(), None, &["legacy-ignored"], &visible).unwrap();
    assert_eq!(ignored.name, "");
    let explicit: TestConfig = load_config_from_visible(
        dir.path(),
        Some(Path::new("legacy-ignored.yml")),
        &["legacy-visible"],
        &visible,
    )
    .unwrap();
    assert_eq!(explicit.name, "explicit-ignored");
}

#[test]
fn test_load_config_multiple_error() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("test.yaml"), "name: a\n").unwrap();
    fs::write(dir.path().join("test.json"), "{\"name\": \"b\"}").unwrap();

    let err = load_config::<TestConfig>(dir.path(), None, &["test"])
        .err()
        .unwrap();
    assert!(err.to_string().contains("multiple config files found"));
}

#[test]
fn test_parse_config_jsonc() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test.jsonc");
    fs::write(&config_path, "{\n  // comment\n  \"name\": \"jsonc\"\n}").unwrap();

    let config: TestConfig = load_config(dir.path(), None, &["test"]).unwrap();
    assert_eq!(config.name, "jsonc");
}

#[test]
fn test_resolve_absolute() {
    let root = Path::new("/root");
    let path = Path::new("/abs/path");
    assert_eq!(resolve(root, path), path.to_path_buf());
}

#[test]
fn test_load_config_explicit_missing() {
    let dir = tempdir().unwrap();
    let err = load_config::<TestConfig>(dir.path(), Some(Path::new("missing.yaml")), &["test"])
        .err()
        .unwrap();
    assert!(err.to_string().contains("config file does not exist"));
}

#[test]
fn test_load_config_read_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.yaml");
    fs::create_dir(&path).unwrap(); // Dir instead of file will cause read error

    // Automatic discovery only considers visible files. Use an explicit path
    // to retain coverage for propagating a read error from a directory.
    let err = load_config::<TestConfig>(dir.path(), Some(Path::new("test.yaml")), &["test"])
        .err()
        .unwrap();
    assert!(err.to_string().contains("directory") || err.to_string().contains("failed"));
}

#[test]
fn test_parse_config_jsonc_error() {
    let err = parse_config::<TestConfig>("{\n  \"name\": \n}", Path::new("test.jsonc"))
        .err()
        .unwrap();
    assert!(err.to_string().contains("Unexpected close brace"));

    let err = parse_config::<TestConfig>("", Path::new("test.jsonc"))
        .err()
        .unwrap();
    assert!(err.to_string().contains("failed to parse"));
}

#[test]
fn test_parse_config_unsupported_extension() {
    let err = parse_config::<TestConfig>("", Path::new("test.toml"))
        .err()
        .unwrap();
    assert!(err
        .to_string()
        .contains("unsupported config file extension"));
}

#[test]
fn test_parse_config_no_extension() {
    let err = parse_config::<TestConfig>("", Path::new("test"))
        .err()
        .unwrap();
    assert!(err
        .to_string()
        .contains("unsupported config file without extension"));
}

#[test]
fn test_parse_config_yaml_parse_error() {
    // Invalid YAML exercises the error branch of `serde_yaml::from_str(source)?` (line 64).
    let err = parse_config::<TestConfig>("name: [unclosed", Path::new("test.yaml"))
        .err()
        .unwrap();
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_parse_config_json_parse_error() {
    // Invalid JSON exercises the error branch of `serde_json::from_str(source)?` (line 65).
    let err = parse_config::<TestConfig>("{\"name\": }", Path::new("test.json"))
        .err()
        .unwrap();
    assert!(!err.to_string().is_empty());
}
