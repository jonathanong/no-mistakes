use super::*;

#[test]
fn glob_normalization_preserves_parent_segments_after_wildcards() {
    let wildcard_parent_glob = build_globset(&["*/../foo".to_string()]).unwrap();

    assert!(wildcard_parent_glob.is_match("pkg/../foo"));
    assert!(!wildcard_parent_glob.is_match("foo"));
}

#[test]
fn swift_load_projects_has_no_config_discovery_or_projects() {
    let root = Path::new("");

    assert!(discovered_config_paths(root, Framework::Swift).is_empty());
    assert!(
        load_config_projects(root, Framework::Swift, "Package.swift", root, "", root)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn dotnet_load_projects_has_no_config_discovery_or_projects() {
    let root = Path::new("");

    assert!(discovered_config_paths(root, Framework::Dotnet).is_empty());
    assert!(
        load_config_projects(root, Framework::Dotnet, "App.csproj", root, "", root)
            .unwrap()
            .is_empty()
    );
}
